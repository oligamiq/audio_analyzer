use anyhow::Context;
use audio_analyzer_core::data::RawDataStreamLayer as _;
use egui::ahash::HashMap;
use egui_snarl::{InPinId, OutPinId, Snarl};
use proc_macro2::TokenStream;
use syn::Ident;

use crate::prelude::nodes::*;

pub fn analysis(snarl: &Snarl<FlowNodes>) -> anyhow::Result<()> {
    let abstract_input_node_id = snarl
        .nodes_ids_data()
        .find(|(_, node)| matches!(node.value, FlowNodes::AbstractInputNode(_)))
        .with_context(|| "No abstract input node found")?
        .0;

    log::info!("Abstract input node id: {:?}", abstract_input_node_id);

    let first_code = match snarl.get_node(abstract_input_node_id) {
        Some(FlowNodes::AbstractInputNode(node)) => {
            log::info!("Abstract input node: {:?}", node);

            let id = abstract_input_node_id.0;
            let name =
                proc_macro2::Ident::new(&format!("out_{}_0", id), proc_macro2::Span::call_site());

            move |outer_code: proc_macro2::TokenStream, next_code: proc_macro2::TokenStream| {
                quote::quote! {
                    let analyzer = |wav_file: &mut audio_analyzer_core::prelude::TestData, sample_rate: u32| {
                        let sample_rate = sample_rate;

                        #outer_code

                        loop {
                            let #name = wav_file.try_recv().unwrap();

                            #next_code
                        }
                    };
                }
            }
        }
        _ => unreachable!(),
    };

    let mut outer_code = proc_macro2::TokenStream::new();
    let mut code = proc_macro2::TokenStream::new();

    let mut next_nodes = get_next_nodes(snarl, abstract_input_node_id);

    let mut checked = vec![abstract_input_node_id];

    fn gen_node_name_out(node_id: &OutPinId) -> proc_macro2::Ident {
        proc_macro2::Ident::new(
            &format!("out_{}_{}", node_id.node.0, node_id.output),
            proc_macro2::Span::call_site(),
        )
    }

    fn gen_node_name_scratch(node: &NodeId, output: usize) -> proc_macro2::Ident {
        proc_macro2::Ident::new(
            &format!("scratch_{}_{}", node.0, output),
            proc_macro2::Span::call_site(),
        )
    }

    fn gen_in_pins_inner(snarl: &Snarl<FlowNodes>, node: &NodeId) -> Vec<(OutPinId, InPinId)> {
        let mut unsorted_in_pins = snarl
            .wires()
            .filter(|(out_pin, _)| &out_pin.node == node)
            .collect::<Vec<_>>();

        unsorted_in_pins.sort_by(|(_, a_in), (_, b_in)| a_in.input.cmp(&b_in.input));

        unsorted_in_pins
    }

    let gen_in_pins = |node: &NodeId| gen_in_pins_inner(snarl, node);

    fn gen_in_pins_with_node_id(snarl: &Snarl<FlowNodes>, node: &NodeId) -> HashMap<usize, Vec<OutPinId>> {
        gen_in_pins_inner(snarl, node)
            .iter()
            .map(|(out_pin, in_pin)| (in_pin.input, out_pin))
            .fold(HashMap::default(), |mut acc, (input, node)| {
                acc.entry(input).or_default().push(node.clone());
                acc
            })
    }

    let gen_in_pins_with_ident = |node: &NodeId| -> HashMap<usize, Vec<syn::Ident>> {
        gen_in_pins(node)
            .iter()
            .map(|(out_pin, in_pin)| {
                let node_name = gen_node_name_out(out_pin);
                (in_pin.input, node_name)
            })
            .fold(HashMap::default(), |mut acc, (input, node_name)| {
                acc.entry(input).or_default().push(node_name);
                acc
            })
    };

    let gen_out_pins = |node: &NodeId| {
        let mut unsorted_out_pins = snarl
            .wires()
            .filter(|(_, in_pin)| &in_pin.node == node)
            .collect::<Vec<_>>();

        unsorted_out_pins.sort_by(|(a_out, _), (b_out, _)| a_out.output.cmp(&b_out.output));

        unsorted_out_pins
    };

    let unique_node_name = |node_id: NodeId| {
        let node_title = snarl
            .get_node(node_id)
            .unwrap()
            .to_as_info()
            .name()
            .to_owned();
        let id = node_id.0;
        proc_macro2::Ident::new(
            &format!("{node_title}_{id}"),
            proc_macro2::Span::call_site(),
        )
    };

    // Retroactively search Input types
    let type_search = |out_pin_id: &OutPinId| {
        let now_node_id = out_pin_id.node;

        fn search_input_type(
            snarl: &Snarl<FlowNodes>,
            now_node_id: OutPinId,
        ) -> anyhow::Result<NodeInfoTypes> {
            let now_node = snarl.get_node(now_node_id.node).unwrap();

            // Only when ExprNode and FrameQueueNode, the type may change.
            // FrameQueueNode is not implemented yet.
            // For other nodes, just follow the same path.
            if let FlowNodes::ExprNode(expr_node) = now_node {
                let in_pins = gen_in_pins_with_node_id(&snarl, &now_node_id.node).get(&0).unwrap().to_owned();
                let first_type = search_input_type(&snarl, in_pins[0].clone())?;

                // Check to see if two nodes are on the input
                if in_pins.len() == 2 {
                    let second_type = search_input_type(&snarl, in_pins[1].clone())?;
                    match (first_type, second_type) {
                        (NodeInfoTypes::Number, NodeInfoTypes::Array1F64)
                        | (NodeInfoTypes::Array1F64, NodeInfoTypes::Number) => {
                            Ok(NodeInfoTypes::Array1TupleF64F64)
                        }
                        (NodeInfoTypes::Array1F64, NodeInfoTypes::Array1F64) => {
                            Ok(NodeInfoTypes::Array1TupleF64F64)
                        }
                        _ => {
                            Ok(first_type)
                        }
                    }
                } else {
                    Ok(first_type)
                }
            } else {
                let ty = now_node.to_as_info().output_types()[now_node_id.output].clone();
                if ty == NodeInfoTypes::AnyInput {
                    panic!("AnyInput is not supported");
                } else if ty == NodeInfoTypes::AnyOutput {
                    let in_pins = gen_in_pins_with_node_id(&snarl, &now_node_id.node).get(&0).unwrap().to_owned();

                    match now_node {
                        FlowNodes::FrameBufferNode(frame_buffer_node) => match frame_buffer_node {
                            FrameBufferNode::FrameQueueNode(_) => unimplemented!(),
                            FrameBufferNode::CycleBufferNode(_) => {
                                let second_type = search_input_type(&snarl, in_pins[1].clone())?;
                                Ok(second_type)
                            },
                        }
                        _ => unreachable!(),
                    }
                } else {
                    Ok(ty)
                }
            }
        }

        search_input_type(&snarl, out_pin_id.clone()).unwrap()
    };

    log::info!("Output input node id: {:?}", &next_nodes);

    loop {
        let node = {
            let (out_pin, _) = match next_nodes.pop() {
                Some(node) => node,
                None => break,
            };

            let node = out_pin.node;

            if checked.contains(&node) {
                continue;
            }
            checked.push(node);

            node
        };

        match snarl.get_node(node).unwrap() {
            FlowNodes::AbstractInputNode(_) => {
                unreachable!("Double abstract input node");
            }
            FlowNodes::LayerNodes(layer_nodes) => match layer_nodes {
                LayerNodes::STFTLayer(stftlayer_node) => {
                    let node_name = unique_node_name(node);
                    let node_name_fft_size = Ident::new(
                        &format!("{}_fft_size", node_name),
                        proc_macro2::Span::call_site(),
                    );
                    let node_name_hop_size = Ident::new(
                        &format!("{}_hop_size", node_name),
                        proc_macro2::Span::call_site(),
                    );

                    outer_code.extend(quote::quote! {
                        // calculator
                        let mut #node_name = audio_analyzer_core::prelude::ToSpectrogramLayer::new(
                            audio_analyzer_core::prelude::FftConfig::default()
                        );

                        // keep the state
                        let mut #node_name_fft_size = #node_name.fft_size;
                        let mut #node_name_hop_size = #node_name.hop_size;
                    });

                    let in_node_names = gen_in_pins_with_ident(&node);

                    let fft_size: TokenStream = in_node_names
                        .get(&0)
                        .map(|v| v.first().map(|v| quote::quote! { #v }))
                        .flatten()
                        .unwrap_or_else(|| {
                            let num = stftlayer_node.fft_size.get();
                            quote::quote! { #num }
                        })
                        .to_owned();
                    let hop_size: TokenStream = in_node_names
                        .get(&1)
                        .map(|v| v.first().map(|v| quote::quote! { #v }))
                        .flatten()
                        .unwrap_or_else(|| {
                            let num = stftlayer_node.hop_size.get();
                            quote::quote! { #num }
                        })
                        .to_owned();
                    let in_data = in_node_names.get(&2).unwrap().first().unwrap().to_owned();

                    let out_data = gen_node_name_scratch(&node, 0);

                    code.extend(quote::quote! {
                        let #out_data = #node_name.through_inner(#in_data);

                        if #node_name_fft_size != #fft_size || #node_name_hop_size != #hop_size {
                            #node_name_fft_size = #fft_size;
                            #node_name_hop_size = #hop_size;

                            #node_name = audio_analyzer_core::prelude::ToSpectrogramLayer::new(
                                audio_analyzer_core::prelude::FftConfig {
                                    fft_size: #fft_size,
                                    hop_size: #hop_size,
                                }
                            );
                        }
                    });
                }
                LayerNodes::MelLayer(mel_layer_node) => todo!(),
                LayerNodes::SpectrogramDensityLayer(spectrogram_density_layer_node) => todo!(),
            },
            FlowNodes::ConfigNodes(config_nodes) => match config_nodes {
                ConfigNodes::NumberNode(number_node) => {
                    let number = number_node.number.get();
                    let out_data = gen_node_name_scratch(&node, 0);

                    code.extend(quote::quote! {
                        let #out_data = #number;
                    });
                }
            },
            FlowNodes::DataInspectorNode(_) => {
                log::info!("DataInspectorNode is ignored");
            }
            FlowNodes::ExprNode(expr_nodes) => {
                let in_pins = gen_in_pins_with_node_id(&snarl, &node);
                let ty = type_search(&in_pins.get(&0).unwrap().first().unwrap());

                
            }
            FlowNodes::FrameBufferNode(frame_buffer_node) => todo!(),
            FlowNodes::FrequencyNodes(frequency_nodes) => todo!(),
            FlowNodes::FilterNodes(filter_nodes) => todo!(),
            FlowNodes::IterNodes(iter_nodes) => todo!(),
            FlowNodes::LpcNodes(lpc_nodes) => todo!(),
            FlowNodes::UnknownNode(unknown_node) => todo!(),
        }
    }

    Ok(())
}

fn get_next_nodes(snarl: &Snarl<FlowNodes>, node: NodeId) -> Vec<(OutPinId, InPinId)> {
    let mut next_nodes: Vec<(OutPinId, InPinId)> = snarl
        .wires()
        .filter(|(out, _)| out.node == node)
        .collect::<Vec<_>>();

    next_nodes.sort_by(|(_, a_out), (_, b_out)| b_out.input.cmp(&a_out.input));

    next_nodes
}
