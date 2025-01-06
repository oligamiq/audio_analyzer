use anyhow::Context;
use audio_analyzer_core::data::RawDataStreamLayer as _;
use egui::ahash::HashMap;
use egui_snarl::{InPinId, Node, OutPinId, Snarl};
use fasteval3::{Compiler, Evaler, Instruction};
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

    fn get_expr_ret(
        eval_str: String,
    ) -> crate::libs::nodes::expr::Ret {
        let parser = fasteval3::Parser::new();
        let mut slab = fasteval3::Slab::new();
        let parsed = parser.parse(&eval_str, &mut slab.ps).unwrap();
        let compiled = parsed.from(&slab.ps).compile(&slab.ps, &mut slab.cs, &mut fasteval3::EmptyNamespace);
        let ret: crate::libs::nodes::expr::Ret = match compiled {
            Instruction::IFunc {name, args} => {
                match name.as_str() {
                    "tuple" => crate::libs::nodes::expr::Ret::Tuple(vec![0.; args.len()]),
                    "complex" => crate::libs::nodes::expr::Ret::Complex(0., 0.),
                    _ => unreachable!("Unknown function")
                }
            }
            _ => unreachable!("Unknown instruction")
        };
        ret
    }

    fn get_expr_out_type (
        snarl: &Snarl<FlowNodes>,
        node_id: &NodeId,
        input_ty: NodeInfoTypes,
    ) -> NodeInfoTypes {
        let node = snarl.get_node(*node_id).unwrap();
        let (eval_str, outputs_num) = match node {
            FlowNodes::ExprNode(expr_node) => (expr_node.expr.clone(), expr_node.outputs_num.get()),
            _ => unreachable!(),
        };
        match input_ty {
            NodeInfoTypes::Number if outputs_num == 1 => NodeInfoTypes::Number,
            NodeInfoTypes::Number if outputs_num == 2 => {
                let ret = get_expr_ret(eval_str);
                match ret {
                    crate::libs::nodes::expr::Ret::Tuple(..) => NodeInfoTypes::Array1TupleF64F64,
                    crate::libs::nodes::expr::Ret::Complex(..) => NodeInfoTypes::Array1ComplexF64,
                }
            },
            NodeInfoTypes::Array1F64 | NodeInfoTypes::Array1TupleF64F64 | NodeInfoTypes::Array1ComplexF64 if outputs_num == 1 => NodeInfoTypes::Array1F64,
            NodeInfoTypes::Array1F64 | NodeInfoTypes::Array1TupleF64F64 | NodeInfoTypes::Array1ComplexF64 => {
                let ret = get_expr_ret(eval_str);
                match ret {
                    crate::libs::nodes::expr::Ret::Tuple(..) => NodeInfoTypes::Array1TupleF64F64,
                    crate::libs::nodes::expr::Ret::Complex(..) => NodeInfoTypes::Array1ComplexF64,
                }
            },
            _ => unimplemented!("Unknown type")
        }
    }

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
            if let FlowNodes::ExprNode(..) = now_node {
                let in_pins = gen_in_pins_with_node_id(&snarl, &now_node_id.node).get(&0).unwrap().to_owned();
                let first_type = search_input_type(&snarl, in_pins[0].clone())?;

                // Check to see if two nodes are on the input
                let in_ty = if in_pins.len() == 2 {
                    let second_type = search_input_type(&snarl, in_pins[1].clone())?;
                    match (first_type, second_type) {
                        (NodeInfoTypes::Number, NodeInfoTypes::Array1F64)
                        | (NodeInfoTypes::Array1F64, NodeInfoTypes::Number) => {
                            NodeInfoTypes::Array1TupleF64F64
                        }
                        (NodeInfoTypes::Array1F64, NodeInfoTypes::Array1F64) => {
                            NodeInfoTypes::Array1TupleF64F64
                        }
                        _ => {
                            first_type
                        }
                    }
                } else {
                    first_type
                };

                Ok(get_expr_out_type(snarl, &now_node_id.node, in_ty))
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
                let first_ty = type_search(&in_pins.get(&0).unwrap().first().unwrap());
                let second_ty = in_pins.get(&0).unwrap().get(1).map(|v| type_search(v));

                let outputs_num = expr_nodes.outputs_num.get();

                let out_data = gen_node_name_scratch(&node, 0);

                let ret = get_expr_ret(expr_nodes.expr.clone());

                let first_in_pin_ident = gen_node_name_out(in_pins.get(&0).unwrap().first().unwrap());
                let second_in_pin_ident = in_pins.get(&0).unwrap().get(1).map(|v| gen_node_name_out(v));

                let (in_ty, translator) = if in_pins.len() == 2 {
                    match (first_ty, second_ty.unwrap()) {
                        (NodeInfoTypes::Number, NodeInfoTypes::Array1F64) => {
                            (NodeInfoTypes::Array1TupleF64F64, quote::quote! {
                                {
                                    #second_in_pin_ident
                                        .into_iter()
                                        .map(|v| (#first_in_pin_ident, v))
                                        .collect::<Array1<(f64, f64)>>()
                                }
                            })
                        }
                        (NodeInfoTypes::Array1F64, NodeInfoTypes::Number) => {
                            (NodeInfoTypes::Array1TupleF64F64, quote::quote! {
                                {
                                    #first_in_pin_ident
                                        .into_iter()
                                        .map(|v| (v, #second_in_pin_ident))
                                        .collect::<Array1<(f64, f64)>>()
                                }
                            })
                        }
                        (NodeInfoTypes::Array1F64, NodeInfoTypes::Array1F64) => {
                            (NodeInfoTypes::Array1TupleF64F64, quote::quote! {
                                {
                                    let is_first_longer = #first_in_pin_ident.len() > #second_in_pin_ident.len();

                                    if is_first_longer {
                                        #first_in_pin_ident
                                            .slice_move(ndarray::s![..#second_in_pin_ident.len()])
                                            .into_iter()
                                            .zip(#second_in_pin_ident.into_iter())
                                            .collect::<Array1<(f64, f64)>>()
                                    } else {
                                        #first_in_pin_ident
                                            .slice_move(ndarray::s![..#first_in_pin_ident.len()])
                                            .into_iter()
                                            .zip(#first_in_pin_ident.into_iter())
                                            .collect::<Array1<(f64, f64)>>()
                                    }
                                }
                            })
                        }
                        _ => unimplemented!("Unknown type"),
                    }
                } else {
                    (first_ty, quote::quote! { #first_in_pin_ident })
                };

                let translator = {
                    let unique_tmp_name = proc_macro2::Ident::new(
                        &format!("tmp_{}", unique_node_name(node)),
                        proc_macro2::Span::call_site(),
                    );

                    code.extend(quote::quote! {
                        let #unique_tmp_name = #translator;
                    });

                    unique_tmp_name
                };

                let normal_eval = expr_nodes.expr.clone();
                let striped_eval = normal_eval.replace(" ", "");
                let rm_tuple = |s: &str| {
                    assert!(s.starts_with("tuple("));
                    assert!(s.ends_with(")"));
                    s.strip_prefix("tuple").unwrap().to_owned()
                };
                let rm_complex = |s: &str| {
                    assert!(s.starts_with("complex("));
                    assert!(s.ends_with(")"));
                    s.strip_prefix("complex").unwrap().to_owned()
                };
                let eval_alt = match in_ty {
                    NodeInfoTypes::Number if outputs_num == 1 => {
                        quote::quote! {
                            #normal_eval
                        }
                    },
                    NodeInfoTypes::Number if outputs_num == 2 => {
                        match ret {
                            crate::libs::nodes::expr::Ret::Tuple(vec) => {
                                let removed_tuple = rm_tuple(&striped_eval);

                                quote::quote! {
                                    {
                                        let x = #translator;
                                        ndarray::Array1::from(vec![{
                                            let mut (a, b) = #removed_tuple;
                                            (a as f64, b as f64)
                                        }])
                                    }
                                }
                            },
                            crate::libs::nodes::expr::Ret::Complex(_, _) => {
                                let removed_complex = rm_complex(&striped_eval);

                                quote::quote! {
                                    {
                                        let x = #translator;
                                        ndarray::Array1::from(vec![{
                                            let mut (a, b) = #removed_complex;
                                            num_complex::Complex::new(a as f64, b as f64)
                                        }])
                                    }
                                }
                            },
                        }
                    },
                    NodeInfoTypes::Array1F64 if outputs_num == 1 => {
                        quote::quote! {
                            {
                                #translator
                                    .iter()
                                    .map(|x| {
                                        #normal_eval as f64
                                    })
                                    .collect::<Array1<f64>>()
                            }
                        }
                    },
                    NodeInfoTypes::Array1F64 if outputs_num == 2 => {
                        match ret {
                            crate::libs::nodes::expr::Ret::Tuple(..) => {
                                let removed_tuple = rm_tuple(&striped_eval);

                                quote::quote! {
                                    {
                                        #translator
                                            .into_iter()
                                            .map(|x| {
                                                let mut (a, b) = #removed_tuple;
                                                (a as f64, b as f64)
                                            })
                                            .collect::<Array1<(f64, f64)>>()
                                    }
                                }
                            },
                            crate::libs::nodes::expr::Ret::Complex(..) => {
                                let removed_complex = rm_complex(&striped_eval);

                                quote::quote! {
                                    {
                                        #translator
                                            .into_iter()
                                            .map(|x| {
                                                let mut (a, b) = #removed_complex;
                                                num_complex::Complex::new(a as f64, b as f64)
                                            })
                                            .collect::<Array1<num_complex::Complex<f64>>>()
                                    }
                                }
                            },
                        }
                    },
                    NodeInfoTypes::Array1TupleF64F64 if outputs_num == 1 => {
                        quote::quote! {
                            {
                                #translator
                                    .into_iter()
                                    .map(|(x, y)| {
                                        #normal_eval as f64
                                    })
                                    .collect::<Array1<f64>>()
                            }
                        }
                    },
                    NodeInfoTypes::Array1TupleF64F64 if outputs_num == 2 => {
                        match ret {
                            crate::libs::nodes::expr::Ret::Tuple(..) => {
                                let removed_tuple = rm_tuple(&striped_eval);

                                quote::quote! {
                                    {
                                        #translator
                                            .into_iter()
                                            .map(|(x, y)| {
                                                let mut (a, b) = #removed_tuple;
                                                (a as f64, b as f64)
                                            })
                                            .collect::<Array1<(f64, f64)>>()
                                    }
                                }
                            },
                            crate::libs::nodes::expr::Ret::Complex(..) => {
                                let removed_complex = rm_complex(&striped_eval);

                                quote::quote! {
                                    {
                                        #translator
                                            .into_iter()
                                            .map(|(x, y)| {
                                                let mut (a, b) = #removed_complex;
                                                num_complex::Complex::new(a as f64, b as f64)
                                            })
                                            .collect::<Array1<num_complex::Complex<f64>>>()
                                    }
                                }
                            },
                        }
                    },
                    NodeInfoTypes::Array1ComplexF64 if outputs_num == 1 => {
                        quote::quote! {
                            {
                                #translator
                                    .into_iter()
                                    .map(|num_complex::Complex { re: x, im: y }| {
                                        #normal_eval as f64
                                    })
                                    .collect::<Array1<f64>>()
                            }
                        }
                    },
                    NodeInfoTypes::Array1ComplexF64 if outputs_num == 2 => {
                        match ret {
                            crate::libs::nodes::expr::Ret::Tuple(..) => {
                                let removed_tuple = rm_tuple(&striped_eval);

                                quote::quote! {
                                    {
                                        #translator
                                            .into_iter()
                                            .map(|num_complex::Complex { re: x, im: y }| {
                                                let mut (a, b) = #removed_tuple;
                                                (a as f64, b as f64)
                                            })
                                            .collect::<Array1<(f64, f64)>>()
                                    }
                                }
                            },
                            crate::libs::nodes::expr::Ret::Complex(..) => {
                                let removed_complex = rm_complex(&striped_eval);

                                quote::quote! {
                                    {
                                        #translator
                                            .into_iter()
                                            .map(|num_complex::Complex { re: x, im: y }| {
                                                let mut (a, b) = #removed_complex;
                                                num_complex::Complex::new(a as f64, b as f64)
                                            })
                                            .collect::<Array1<num_complex::Complex<f64>>>()
                                    }
                                }
                            },
                        }
                    },
                    _ => unimplemented!("Unknown type")
                };

                code.extend(quote::quote! {
                    let #out_data = #eval_alt;
                });
            }
            FlowNodes::FrameBufferNode(frame_buffer_node) => match frame_buffer_node {
                FrameBufferNode::FrameQueueNode(frame_queue_node) => todo!(),
                FrameBufferNode::CycleBufferNode(cycle_buffer_node) => {
                    let in_pins = gen_in_pins_with_node_id(&snarl, &node);
                    let in_ty = type_search(&in_pins.get(&1).unwrap().first().unwrap());

                    let node_name = unique_node_name(node);
                    let node_name_buffer = Ident::new(
                        &format!("{}_buffer", node_name),
                        proc_macro2::Span::call_site(),
                    );
                    let node_name_buffer_size = Ident::new(
                        &format!("{}_buffer_size", node_name),
                        proc_macro2::Span::call_site(),
                    );

                    match in_ty {
                        NodeInfoTypes::Array1F64 => {
                            // let size = cycle_buffer_node.len.get();
                            let size: TokenStream = in_pins
                                .get(&1)
                                .map(|v| v.first().map(|v| {
                                    let ident = gen_node_name_out(v);
                                    quote::quote! { #ident }
                                }))
                                .flatten()
                                .unwrap_or_else(|| {
                                    let num = cycle_buffer_node.len.get();
                                    quote::quote! { #num }
                                })
                                .to_owned();

                            outer_code.extend(quote::quote! {
                                // calculator
                                let mut #node_name = Array1::<f64>::zeros(#size);
                            });
                        },
                        NodeInfoTypes::Array1TupleF64F64 => todo!(),
                        NodeInfoTypes::Array2F64 => todo!(),
                        NodeInfoTypes::Array1ComplexF64 => todo!(),
                        NodeInfoTypes::AnyInput => todo!(),
                        NodeInfoTypes::AnyOutput => todo!(),
                    }

                    let in_node_names = gen_in_pins_with_ident(&node);

                    let buffer_size: TokenStream = in_node_names
                        .get(&0)
                        .map(|v| v.first().map(|v| quote::quote! { #v }))
                        .flatten()
                        .unwrap_or_else(|| {
                            let num = cycle_buffer_node.buffer_size.get();
                            quote::quote! { #num }
                        })
                        .to_owned();
                    let in_data = in_node_names.get(&1).unwrap().first().unwrap().to_owned();

                    let out_data = gen_node_name_scratch(&node, 0);

                    code.extend(quote::quote! {
                        let #out_data = #node_name.through_inner(#in_data);

                        if #node_name_buffer_size != #buffer_size {
                            #node_name_buffer_size = #buffer_size;

                            #node_name = audio_analyzer_core::prelude::CycleBufferLayer::new(
                                audio_analyzer_core::prelude::CycleBufferConfig {
                                    buffer_size: #buffer_size,
                                }
                            );
                        }
                    });
                },
            },
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
