use anyhow::Context;
use audio_analyzer_core::data::RawDataStreamLayer as _;
use egui::ahash::HashMap;
use egui_snarl::{InPinId, Node, OutPinId, Snarl};
use fasteval3::{Compiler, Evaler, Instruction};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::Ident;

use crate::prelude::{nodes::*, snarl};

pub fn analysis(snarl: &Snarl<FlowNodes>) -> anyhow::Result<TokenStream> {
    let abstract_input_node_id = snarl
        .nodes_ids_data()
        .find(|(_, node)| matches!(node.value, FlowNodes::AbstractInputNode(_)))
        .with_context(|| "No abstract input node found")?
        .0;

    // log::info!("Abstract input node id: {:?}", abstract_input_node_id);

    println!("Abstract input node id: {:?}", abstract_input_node_id);

    let abstract_input_node = snarl.get_node(abstract_input_node_id);
    let first_code = match abstract_input_node {
        Some(FlowNodes::AbstractInputNode(node)) => {
            // log::info!("Abstract input node: {:?}", node);

            println!("call first_code: AbstractInputNode");

            let title = node_title(abstract_input_node.unwrap());

            let id = abstract_input_node_id.0;
            let name = proc_macro2::Ident::new(
                &format!("{title}_out_{}_0", id),
                proc_macro2::Span::call_site(),
            );

            let sample_rate_ident = proc_macro2::Ident::new(
                &format!("{title}_out_{}_1", id),
                proc_macro2::Span::call_site(),
            );

            move |outer_code: &proc_macro2::TokenStream,
                  next_code: &proc_macro2::TokenStream,
                  hop_size: &proc_macro2::TokenStream| {
                quote::quote! {
                    pub fn analyzer(wav_file: &mut audio_analyzer_core::prelude::TestData, sample_rate: u32) {
                        use audio_analyzer_core::data::RawDataStreamLayer as _;
                        use crate::presets::*;

                        let sample_rate = sample_rate as f64;

                        let hop_size = #hop_size;

                        #outer_code

                        let mut buffer = Vec::new();
                        for frame in wav_file.try_recv() {
                            buffer.extend(frame);

                            while buffer.len() >= hop_size {
                                let #name: ndarray::Array1<f64> = buffer.drain(..hop_size).collect::<ndarray::Array1<_>>();
                                if #name.len() == 0 {
                                    break;
                                }

                                let #sample_rate_ident = sample_rate;

                                #next_code
                            }
                        }
                    }
                }
            }
        }
        _ => unreachable!(),
    };

    let mut outer_code = proc_macro2::TokenStream::new();
    let mut code = proc_macro2::TokenStream::new();
    let mut hop_size = proc_macro2::TokenStream::new();

    fn node_title(node: &FlowNodes) -> String {
        use convert_case::{Case, Casing};

        node.to_as_info().name().to_owned().to_case(Case::Snake)
    }

    fn gen_node_name_out(snarl: &Snarl<FlowNodes>, node_id: &OutPinId) -> proc_macro2::Ident {
        let title = node_title(snarl.get_node(node_id.node).unwrap());

        proc_macro2::Ident::new(
            &format!("{title}_out_{}_{}", node_id.node.0, node_id.output),
            proc_macro2::Span::call_site(),
        )
    }

    fn gen_node_name_scratch(snarl: &Snarl<FlowNodes>, node: &NodeId, output: usize) -> Ident {
        let title = node_title(snarl.get_node(*node).unwrap());

        proc_macro2::Ident::new(
            &format!("{title}_out_{}_{}", node.0, output),
            proc_macro2::Span::call_site(),
        )
    }

    fn gen_in_pins_inner(snarl: &Snarl<FlowNodes>, node: &NodeId) -> Vec<(OutPinId, InPinId)> {
        let mut unsorted_in_pins = snarl
            .wires()
            .filter(|(_, in_pin)| &in_pin.node == node)
            .collect::<Vec<_>>();

        unsorted_in_pins.sort_by(|(_, a_in), (_, b_in)| a_in.input.cmp(&b_in.input));

        unsorted_in_pins
    }

    let gen_in_pins = |node: &NodeId| gen_in_pins_inner(snarl, node);

    fn gen_in_pins_with_node_id(
        snarl: &Snarl<FlowNodes>,
        node: &NodeId,
    ) -> HashMap<usize, Vec<OutPinId>> {
        gen_in_pins_inner(snarl, node)
            .iter()
            .map(|(out_pin, in_pin)| (in_pin.input, out_pin))
            .fold(HashMap::default(), |mut acc, (input, node)| {
                acc.entry(input).or_default().push(node.clone());
                acc
            })
    }

    fn gen_in_pins_with_ident(
        snarl: &Snarl<FlowNodes>,
        node: &NodeId,
    ) -> HashMap<usize, Vec<Ident>> {
        gen_in_pins_inner(snarl, node)
            .iter()
            .map(|(out_pin, in_pin)| {
                let node_name = gen_node_name_out(snarl, out_pin);
                (in_pin.input, node_name)
            })
            .fold(HashMap::default(), |mut acc, (input, node_name)| {
                acc.entry(input).or_default().push(node_name);
                acc
            })
    }

    // let gen_out_pins = |node: &NodeId| {
    //     let mut unsorted_out_pins = snarl
    //         .wires()
    //         .filter(|(_, in_pin)| &in_pin.node == node)
    //         .collect::<Vec<_>>();

    //     unsorted_out_pins.sort_by(|(a_out, _), (b_out, _)| a_out.output.cmp(&b_out.output));

    //     unsorted_out_pins
    // };

    fn unique_node_name(snarl: &Snarl<FlowNodes>, node_id: NodeId) -> proc_macro2::Ident {
        let node_title = node_title(snarl.get_node(node_id).unwrap());
        let id = node_id.0;
        proc_macro2::Ident::new(
            &format!("{node_title}_{id}"),
            proc_macro2::Span::call_site(),
        )
    }

    fn get_expr_ret(eval_str: String) -> Option<crate::libs::nodes::expr::Ret> {
        let parser = fasteval3::Parser::new();
        let mut slab = fasteval3::Slab::new();
        let parsed = parser.parse(&eval_str, &mut slab.ps).unwrap();
        let compiled =
            parsed
                .from(&slab.ps)
                .compile(&slab.ps, &mut slab.cs, &mut fasteval3::EmptyNamespace);
        let ret: Option<crate::libs::nodes::expr::Ret> = match compiled {
            Instruction::IFunc { name, args } => match name.as_str() {
                "tuple" => Some(crate::libs::nodes::expr::Ret::Tuple(vec![0.; args.len()])),
                "complex" => Some(crate::libs::nodes::expr::Ret::Complex(0., 0.)),
                _ => unreachable!("Unknown function"),
            },
            _ => {
                if eval_str.contains("complex") {
                    Some(crate::libs::nodes::expr::Ret::Complex(0., 0.))
                } else if eval_str.contains("tuple") {
                    Some(crate::libs::nodes::expr::Ret::Tuple(vec![0.; 2]))
                } else {
                    None
                }
            }
        };
        ret
    }

    fn get_expr_out_type(
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
                    Some(crate::libs::nodes::expr::Ret::Tuple(..)) => {
                        NodeInfoTypes::Array1TupleF64F64
                    }
                    Some(crate::libs::nodes::expr::Ret::Complex(..)) => {
                        NodeInfoTypes::Array1ComplexF64
                    }
                    _ => unreachable!("Unknown type"),
                }
            }
            NodeInfoTypes::Array1F64
            | NodeInfoTypes::Array1TupleF64F64
            | NodeInfoTypes::Array1ComplexF64
                if outputs_num == 1 =>
            {
                NodeInfoTypes::Array1F64
            }
            NodeInfoTypes::Array1F64
            | NodeInfoTypes::Array1TupleF64F64
            | NodeInfoTypes::Array1ComplexF64 => {
                let ret = get_expr_ret(eval_str);
                match ret {
                    Some(crate::libs::nodes::expr::Ret::Tuple(..)) => {
                        NodeInfoTypes::Array1TupleF64F64
                    }
                    Some(crate::libs::nodes::expr::Ret::Complex(..)) => {
                        NodeInfoTypes::Array1ComplexF64
                    }
                    _ => unreachable!("Unknown type"),
                }
            }
            _ => unimplemented!("Unknown type"),
        }
    }

    // Retroactively search Input types
    fn type_search(snarl: &Snarl<FlowNodes>, out_pin_id: &OutPinId) -> NodeInfoTypes {
        fn search_input_type(
            snarl: &Snarl<FlowNodes>,
            now_node_id: OutPinId,
        ) -> anyhow::Result<NodeInfoTypes> {
            let now_node = snarl.get_node(now_node_id.node).unwrap();

            // Only when ExprNode and FrameQueueNode, the type may change.
            // FrameQueueNode is not implemented yet.
            // For other nodes, just follow the same path.
            if let FlowNodes::ExprNode(..) = now_node {
                let in_pins = gen_in_pins_with_node_id(&snarl, &now_node_id.node)
                    .get(&0)
                    .unwrap()
                    .to_owned();

                println!("in_pins: {:?}", in_pins);

                let first_type = search_input_type(&snarl, in_pins.get(0).unwrap().clone())?;

                // Check to see if two nodes are on the input
                let in_ty = if in_pins.len() == 2 {
                    let second_type = search_input_type(&snarl, in_pins.get(1).unwrap().clone())?;
                    match (first_type, second_type) {
                        (NodeInfoTypes::Number, NodeInfoTypes::Array1F64)
                        | (NodeInfoTypes::Array1F64, NodeInfoTypes::Number) => {
                            NodeInfoTypes::Array1TupleF64F64
                        }
                        (NodeInfoTypes::Array1F64, NodeInfoTypes::Array1F64) => {
                            NodeInfoTypes::Array1TupleF64F64
                        }
                        _ => first_type,
                    }
                } else {
                    first_type
                };

                Ok(get_expr_out_type(snarl, &now_node_id.node, in_ty))
            } else {
                let ty = now_node
                    .to_as_info()
                    .output_types()
                    .get(now_node_id.output)
                    .unwrap()
                    .clone();
                if ty == NodeInfoTypes::AnyInput {
                    panic!("AnyInput is not supported");
                } else if ty == NodeInfoTypes::AnyOutput {
                    let in_pins = gen_in_pins_with_node_id(&snarl, &now_node_id.node);

                    match now_node {
                        FlowNodes::FrameBufferNode(frame_buffer_node) => match frame_buffer_node {
                            FrameBufferNode::FrameQueueNode(_) => unimplemented!(),
                            FrameBufferNode::CycleBufferNode(_) => {
                                let second_type = search_input_type(
                                    &snarl,
                                    in_pins.get(&1).unwrap().first().unwrap().clone(),
                                )?;
                                Ok(second_type)
                            }
                        },
                        _ => unreachable!(),
                    }
                } else {
                    Ok(ty)
                }
            }
        }

        log::info!("Output input node id: {:?}", out_pin_id);

        search_input_type(&snarl, out_pin_id.clone()).unwrap()
    }

    let mut before_nodes = gen_in_pins_with_node_id(snarl, &abstract_input_node_id);

    if let Some(in_pins) = before_nodes.get(&0) {
        let out_pin = in_pins.first().unwrap();

        let mut all_nodes = get_ancestors(
            snarl,
            &abstract_input_node_id,
            &vec![],
            &abstract_input_node_id,
        );

        fn get_ancestors(
            snarl: &Snarl<FlowNodes>,
            node_id: &NodeId,
            checked: &Vec<NodeId>,
            abstract_input_node_id: &NodeId,
        ) -> Vec<NodeId> {
            println!(
                "checked: {:?}",
                checked
                    .into_iter()
                    .map(|v| node_title(snarl.get_node(*v).unwrap()))
                    .collect::<Vec<_>>()
            );

            gen_in_pins_with_node_id(snarl, &node_id)
                .into_iter()
                .flat_map(|(_, out_pin)| {
                    out_pin.into_iter().flat_map(|out_pin| {
                        if out_pin.node == *abstract_input_node_id {
                            vec![]
                        } else if checked.contains(&out_pin.node) {
                            vec![]
                        } else {
                            get_ancestors(
                                snarl,
                                &out_pin.node,
                                &[checked.to_owned(), vec![out_pin.node]].concat(),
                                abstract_input_node_id,
                            )
                        }
                    })
                })
                .chain(vec![node_id.clone()])
                .collect::<Vec<_>>()
        }

        println!(
            "all_nodes: {:?}",
            all_nodes
                .into_iter()
                .map(|v| node_title(snarl.get_node(v).unwrap()))
                .collect::<Vec<_>>()
        );
    } else {
        hop_size.extend({
            let abstract_input_node = snarl.get_node(abstract_input_node_id).unwrap();
            let hop_size = match abstract_input_node {
                FlowNodes::AbstractInputNode(node) => node.hop_size.get(),
                _ => unreachable!(),
            };
            quote::quote! { #hop_size }
        });
    }

    let mut all_nodes = snarl
        .node_ids()
        .map(|(id, _)| {
            let before_nodes = gen_in_pins(&id);
            (
                id,
                before_nodes
                    .iter()
                    .map(|(out_pin, _)| out_pin.clone())
                    .collect::<Vec<_>>(),
            )
        })
        .filter(|(id, _)| *id != abstract_input_node_id)
        .collect::<HashMap<_, _>>();

    loop {
        let node = {
            let next_node = all_nodes.iter().find(|(_, before_nodes)| {
                before_nodes
                    .iter()
                    .all(|out_pin| !all_nodes.contains_key(&out_pin.node))
            });

            let node = match next_node {
                Some((id, _)) => id,
                None => {
                    log::info!("No more nodes");
                    break;
                }
            }
            .to_owned();

            all_nodes.remove(&node);

            node
        };

        log::info!("Node id: {:?}", node);

        fn gen_code(
            node: &NodeId,
            snarl: &Snarl<FlowNodes>,
            outer_code: &mut proc_macro2::TokenStream,
            code: &mut proc_macro2::TokenStream,
        ) -> anyhow::Result<()> {
            let node = *node;

            match snarl.get_node(node).unwrap() {
                FlowNodes::AbstractInputNode(_) => {
                    return Err(anyhow::anyhow!("AbstractInputNode is not allowed here"));
                }
                FlowNodes::LayerNodes(layer_nodes) => match layer_nodes {
                    LayerNodes::STFTLayer(stftlayer_node) => {
                        let node_name = unique_node_name(snarl, node);
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
                            let mut #node_name_fft_size = 400usize;
                            let mut #node_name_hop_size = 160usize;
                        });

                        let in_node_names: std::collections::HashMap<
                            usize,
                            Vec<Ident>,
                            egui::ahash::RandomState,
                        > = gen_in_pins_with_ident(snarl, &node);

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

                        let out_data = gen_node_name_scratch(snarl, &node, 0);

                        code.extend(quote::quote! {
                            {
                                let #fft_size = #fft_size as usize;
                                let #hop_size = #hop_size as usize;

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
                            }

                            let #out_data = {
                                let #out_data = match #node_name.through_inner(&#in_data.clone().to_vec()).unwrap().first() {
                                    Some(v) => v.to_owned(),
                                    None => continue,
                                };
                                assert!(!#out_data.is_empty());

                                #out_data
                            };
                        });
                    }
                    LayerNodes::MelLayer(mel_layer_node) => todo!(),
                    LayerNodes::SpectrogramDensityLayer(spectrogram_density_layer_node) => todo!(),
                },
                FlowNodes::ConfigNodes(config_nodes) => match config_nodes {
                    ConfigNodes::NumberNode(number_node) => {
                        let number = number_node.number.get();
                        let out_data = gen_node_name_scratch(snarl, &node, 0);

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
                    let first_ty = type_search(snarl, &in_pins.get(&0).unwrap().first().unwrap());
                    let second_ty = in_pins
                        .get(&0)
                        .unwrap()
                        .get(1)
                        .map(|v| type_search(snarl, v));

                    let outputs_num = expr_nodes.outputs_num.get();

                    let out_data = gen_node_name_scratch(snarl, &node, 0);

                    let ret = get_expr_ret(expr_nodes.expr.clone());

                    let first_in_pin_ident =
                        gen_node_name_out(snarl, in_pins.get(&0).unwrap().first().unwrap());
                    let second_in_pin_ident = in_pins
                        .get(&0)
                        .unwrap()
                        .get(1)
                        .map(|v| gen_node_name_out(snarl, v));

                    let (in_ty, translator) = if in_pins.get(&0).unwrap().len() == 2 {
                        match (first_ty, second_ty.unwrap()) {
                            (NodeInfoTypes::Number, NodeInfoTypes::Array1F64) => (
                                NodeInfoTypes::Array1TupleF64F64,
                                quote::quote! {
                                    {
                                        #second_in_pin_ident
                                            .into_iter()
                                            .map(|v| (#first_in_pin_ident.clone(), v))
                                            .collect::<ndarray::Array1<(f64, f64)>>()
                                    }
                                },
                            ),
                            (NodeInfoTypes::Array1F64, NodeInfoTypes::Number) => (
                                NodeInfoTypes::Array1TupleF64F64,
                                quote::quote! {
                                    {
                                        #first_in_pin_ident.clone()
                                            .into_iter()
                                            .map(|v| (v, #second_in_pin_ident.clone()))
                                            .collect::<ndarray::Array1<(f64, f64)>>()
                                    }
                                },
                            ),
                            (NodeInfoTypes::Array1F64, NodeInfoTypes::Array1F64) => (
                                NodeInfoTypes::Array1TupleF64F64,
                                quote::quote! {
                                    {
                                        let is_first_longer = #first_in_pin_ident.len() as f64 > #second_in_pin_ident.len() as f64;

                                        if is_first_longer {
                                            #first_in_pin_ident
                                                .clone()
                                                .slice_move(ndarray::s![..#second_in_pin_ident.len()])
                                                .into_iter()
                                                .zip(#second_in_pin_ident.clone().into_iter())
                                                .collect::<ndarray::Array1<(f64, f64)>>()
                                        } else {
                                            #first_in_pin_ident
                                                .clone()
                                                .slice_move(ndarray::s![..#first_in_pin_ident.len()])
                                                .into_iter()
                                                .zip(#first_in_pin_ident.clone().into_iter())
                                                .collect::<ndarray::Array1<(f64, f64)>>()
                                        }
                                    }
                                },
                            ),
                            _ => unimplemented!("Unknown type"),
                        }
                    } else {
                        (first_ty, quote::quote! { #first_in_pin_ident })
                    };

                    let translator = {
                        let unique_tmp_name = proc_macro2::Ident::new(
                            &format!("tmp_{}", unique_node_name(snarl, node)),
                            proc_macro2::Span::call_site(),
                        );

                        code.extend(quote::quote! {
                            let #unique_tmp_name = #translator.clone();
                        });

                        unique_tmp_name
                    };

                    let normal_eval = expr_nodes.expr.clone();
                    let striped_eval = normal_eval.replace(" ", "");

                    fn append_f64(expr_: String) -> String {
                        let expr = expr_.clone();
                        // Append `as f64` where you think it is a integer
                        let mut integers = vec![];
                        let mut index = 0;
                        loop {
                            // find the first integer
                            let start = match expr[index..]
                                .find(|c: char| c.is_digit(10))
                                .map(|v| index + v)
                            {
                                Some(v) => v,
                                None => break,
                            };
                            let end = match expr[start..]
                                .find(|c: char| !c.is_digit(10) && c != '.')
                                .map(|v| start + v)
                            {
                                Some(v) => v - 1,
                                None => expr.len() - 1,
                            };

                            let integer = &expr[start..=end];

                            if let Ok(integer) = integer.parse::<i64>() {
                                integers.push((start, end, integer));
                            }

                            index = end + 1;
                        }

                        let mut expr = expr.clone();
                        let mut diff = 0;
                        for (_, end, _) in integers {
                            if end + diff + 1 >= expr.len() {
                                expr.push_str("f64");
                                break;
                            } else {
                                expr.insert_str(end + diff + 1, "f64");
                                diff += "f64".len();
                            }
                        }

                        expr
                    }
                    let rm_tuple = |s: &str| {
                        assert!(s.starts_with("tuple("));
                        assert!(s.ends_with(")"));
                        let expr = s
                            .strip_prefix("tuple")
                            .unwrap()
                            .to_owned()
                            .parse::<TokenStream>()
                            .unwrap();
                        let expr = append_f64(
                            syn::parse2::<syn::Expr>(expr)
                                .unwrap()
                                .to_token_stream()
                                .to_string(),
                        );
                        expr.parse::<TokenStream>().unwrap()
                    };
                    let rm_complex = |s: &str| {
                        assert!(s.starts_with("complex("));
                        assert!(s.ends_with(")"));
                        let expr = s
                            .strip_prefix("complex")
                            .unwrap()
                            .to_owned()
                            .parse::<TokenStream>()
                            .unwrap();
                        let expr = append_f64(
                            syn::parse2::<syn::Expr>(expr)
                                .unwrap()
                                .to_token_stream()
                                .to_string(),
                        );
                        expr.parse::<TokenStream>().unwrap()
                    };
                    let normal_eval = {
                        let normal_eval =
                            append_f64(normal_eval.parse::<TokenStream>().unwrap().to_string());
                        normal_eval.parse::<TokenStream>().unwrap()
                    };
                    let eval_alt = match in_ty {
                        NodeInfoTypes::Number if outputs_num == 1 => {
                            quote::quote! {
                                {
                                    let x = #translator;
                                    #normal_eval
                                }
                            }
                        }
                        NodeInfoTypes::Number if outputs_num == 2 => match ret {
                            Some(crate::libs::nodes::expr::Ret::Tuple(..)) => {
                                let removed_tuple = rm_tuple(&striped_eval);

                                quote::quote! {
                                    {
                                        let x = #translator;
                                        ndarray::Array1::from(vec![{
                                            let (a, b) = #removed_tuple;
                                            (a as f64, b as f64)
                                        }])
                                    }
                                }
                            }
                            Some(crate::libs::nodes::expr::Ret::Complex(..)) => {
                                let removed_complex = rm_complex(&striped_eval);

                                quote::quote! {
                                    {
                                        let x = #translator;
                                        ndarray::Array1::from(vec![{
                                            let (a, b) = #removed_complex;
                                            num_complex::Complex::new(a as f64, b as f64)
                                        }])
                                    }
                                }
                            }
                            _ => unimplemented!("Unknown type"),
                        },
                        NodeInfoTypes::Array1F64 if outputs_num == 1 => {
                            quote::quote! {
                                {
                                    #translator
                                        .iter()
                                        .map(|&x| {
                                            (#normal_eval) as f64
                                        })
                                        .collect::<ndarray::Array1<f64>>()
                                }
                            }
                        }
                        NodeInfoTypes::Array1F64 if outputs_num == 2 => match ret {
                            Some(crate::libs::nodes::expr::Ret::Tuple(..)) => {
                                let removed_tuple = rm_tuple(&striped_eval);

                                quote::quote! {
                                    {
                                        #translator
                                            .into_iter()
                                            .map(|x| {
                                                let (a, b) = #removed_tuple;
                                                (a as f64, b as f64)
                                            })
                                            .collect::<ndarray::Array1<(f64, f64)>>()
                                    }
                                }
                            }
                            Some(crate::libs::nodes::expr::Ret::Complex(..)) => {
                                let removed_complex = rm_complex(&striped_eval);

                                quote::quote! {
                                    {
                                        #translator
                                            .into_iter()
                                            .map(|x| {
                                                let (a, b) = #removed_complex;
                                                num_complex::Complex::new(a as f64, b as f64)
                                            })
                                            .collect::<ndarray::Array1<num_complex::Complex<f64>>>()
                                    }
                                }
                            }
                            _ => unimplemented!("Unknown type"),
                        },
                        NodeInfoTypes::Array1TupleF64F64 if outputs_num == 1 => {
                            quote::quote! {
                                {
                                    #translator
                                        .into_iter()
                                        .map(|(x, y)| {
                                            #normal_eval as f64
                                        })
                                        .collect::<ndarray::Array1<f64>>()
                                }
                            }
                        }
                        NodeInfoTypes::Array1TupleF64F64 if outputs_num == 2 => match ret {
                            Some(crate::libs::nodes::expr::Ret::Tuple(..)) => {
                                let removed_tuple = rm_tuple(&striped_eval);

                                quote::quote! {
                                    {
                                        #translator
                                            .into_iter()
                                            .map(|(x, y)| {
                                                let (a, b) = #removed_tuple;
                                                (a as f64, b as f64)
                                            })
                                            .collect::<ndarray::Array1<(f64, f64)>>()
                                    }
                                }
                            }
                            Some(crate::libs::nodes::expr::Ret::Complex(..)) => {
                                let removed_complex = rm_complex(&striped_eval);

                                quote::quote! {
                                    {
                                        #translator
                                            .into_iter()
                                            .map(|(x, y)| {
                                                let (a, b) = #removed_complex;
                                                num_complex::Complex::new(a as f64, b as f64)
                                            })
                                            .collect::<ndarray::Array1<num_complex::Complex<f64>>>()
                                    }
                                }
                            }
                            _ => unimplemented!("Unknown type"),
                        },
                        NodeInfoTypes::Array1ComplexF64 if outputs_num == 1 => {
                            quote::quote! {
                                {
                                    #translator
                                        .into_iter()
                                        .map(|num_complex::Complex { re: x, im: y }| {
                                            #normal_eval as f64
                                        })
                                        .collect::<ndarray::Array1<f64>>()
                                }
                            }
                        }
                        NodeInfoTypes::Array1ComplexF64 if outputs_num == 2 => match ret {
                            Some(crate::libs::nodes::expr::Ret::Tuple(..)) => {
                                let removed_tuple = rm_tuple(&striped_eval);

                                quote::quote! {
                                    {
                                        #translator
                                            .into_iter()
                                            .map(|num_complex::Complex { re: x, im: y }| {
                                                let (a, b) = #removed_tuple;
                                                (a as f64, b as f64)
                                            })
                                            .collect::<ndarray::Array1<(f64, f64)>>()
                                    }
                                }
                            }
                            Some(crate::libs::nodes::expr::Ret::Complex(..)) => {
                                let removed_complex = rm_complex(&striped_eval);

                                quote::quote! {
                                    {
                                        #translator
                                            .into_iter()
                                            .map(|num_complex::Complex { re: x, im: y }| {
                                                let (a, b) = #removed_complex;
                                                num_complex::Complex::new(a as f64, b as f64)
                                            })
                                            .collect::<ndarray::Array1<num_complex::Complex<f64>>>()
                                    }
                                }
                            }
                            _ => unimplemented!("Unknown type"),
                        },
                        _ => unimplemented!("Unknown type"),
                    };

                    code.extend(quote::quote! {
                        let #out_data = #eval_alt;
                    });
                }
                FlowNodes::FrameBufferNode(frame_buffer_node) => match frame_buffer_node {
                    FrameBufferNode::FrameQueueNode(..) => todo!(),
                    FrameBufferNode::CycleBufferNode(cycle_buffer_node) => {
                        let in_pins = gen_in_pins_with_node_id(&snarl, &node);
                        let in_ty = type_search(snarl, &in_pins.get(&1).unwrap().first().unwrap());

                        let node_name = unique_node_name(snarl, node);

                        let size: TokenStream = in_pins
                            .get(&0)
                            .map(|v| {
                                v.first().map(|v| {
                                    let ident = gen_node_name_out(snarl, v);
                                    quote::quote! { #ident }
                                })
                            })
                            .flatten()
                            .unwrap_or_else(|| {
                                let num = cycle_buffer_node.len.get();
                                quote::quote! { #num }
                            })
                            .to_owned();

                        let out_data = gen_node_name_scratch(snarl, &node, 0);

                        let in_data =
                            gen_node_name_out(snarl, in_pins.get(&1).unwrap().first().unwrap());

                        let array_code = quote::quote! {
                            let #out_data = {
                                let extended = ndarray::concatenate(
                                    ndarray::Axis(0),
                                    &[
                                        #node_name.view(),
                                        #in_data.view(),
                                    ],
                                ).unwrap();

                                let new_len = extended.len() as f64;

                                if new_len > #size {
                                    let diff = (new_len - #size) as usize;
                                    let new_buffer = extended.slice(ndarray::s![diff..]);
                                    #node_name = new_buffer.to_owned();

                                    assert!(new_buffer.len() as f64 == #size);
                                } else {
                                    #node_name = extended.to_owned();
                                }

                                #node_name.clone()
                            };
                        };

                        match in_ty {
                            NodeInfoTypes::Array1F64 => {
                                outer_code.extend(quote::quote! {
                                    let mut #node_name = ndarray::Array1::<f64>::zeros(0);
                                });

                                code.extend(array_code.clone());
                            }
                            NodeInfoTypes::Array1TupleF64F64 => {
                                outer_code.extend(quote::quote! {
                                    let mut #node_name = ndarray::Array1::<(f64, f64)>::zeros(0);
                                });

                                code.extend(array_code.clone());
                            }
                            NodeInfoTypes::Array1ComplexF64 => {
                                outer_code.extend(quote::quote! {
                                    let mut #node_name = ndarray::Array1::<num_complex::Complex<f64>>::zeros(0);
                                });

                                code.extend(array_code.clone());
                            }
                            NodeInfoTypes::Array2F64 => todo!(),
                            _ => unimplemented!("Unknown type"),
                        }
                    }
                },
                FlowNodes::FrequencyNodes(frequency_node) => match frequency_node {
                    FrequencyNodes::FFTNode(_) => {
                        let node_name = unique_node_name(snarl, node);
                        let node_name_fft_size = Ident::new(
                            &format!("{}_fft_size", node_name),
                            proc_macro2::Span::call_site(),
                        );
                        let node_name_scratch_buf = Ident::new(
                            &format!("{}_scratch_buf", node_name),
                            proc_macro2::Span::call_site(),
                        );

                        outer_code.extend(quote::quote! {
                            // keep the state
                            let mut #node_name_fft_size = 400;
                            // calculator
                            let mut #node_name = {
                                let mut planner = rustfft::FftPlanner::new();
                                let fft = planner.plan_fft_forward(#node_name_fft_size);
                                fft
                            };
                            // scratch buffer
                            let mut #node_name_scratch_buf = vec![num_complex::Complex::new(0.0, 0.0); #node_name_fft_size];
                        });

                        let in_node_names: std::collections::HashMap<
                            usize,
                            Vec<Ident>,
                            egui::ahash::RandomState,
                        > = gen_in_pins_with_ident(snarl, &node);

                        let in_data = in_node_names.get(&0).unwrap().first().unwrap().to_owned();
                        let fft_size: TokenStream = quote::quote! { #in_data.len() };

                        let out_data = gen_node_name_scratch(snarl, &node, 0);

                        code.extend(quote::quote! {
                            if #node_name_fft_size != #fft_size {
                                #node_name_fft_size = #fft_size;

                                #node_name = {
                                    let mut planner = rustfft::FftPlanner::new();
                                    let fft = planner.plan_fft_forward(#node_name_fft_size);
                                    fft
                                };
                                #node_name_scratch_buf = vec![num_complex::Complex::new(0.0, 0.0); #node_name_fft_size];
                            }

                            let #out_data = {
                                let mut #out_data = #in_data.clone().to_vec();
                                #node_name.process_with_scratch(#out_data.as_mut_slice(), #node_name_scratch_buf.as_mut_slice());
                                #out_data.into_iter().collect::<ndarray::Array1<num_complex::Complex<f64>>>()
                            };
                        });
                    }
                    FrequencyNodes::IFFTNode(_) => {
                        let node_name = unique_node_name(snarl, node);
                        let node_name_fft_size = Ident::new(
                            &format!("{}_fft_size", node_name),
                            proc_macro2::Span::call_site(),
                        );
                        let node_name_scratch_buf = Ident::new(
                            &format!("{}_scratch_buf", node_name),
                            proc_macro2::Span::call_site(),
                        );

                        outer_code.extend(quote::quote! {
                            // keep the state
                            let mut #node_name_fft_size = 400;
                            // calculator
                            let mut #node_name = {
                                let mut planner = rustfft::FftPlanner::new();
                                let fft = planner.plan_fft_inverse(#node_name_fft_size);
                                fft
                            };
                            // scratch buffer
                            let mut #node_name_scratch_buf = vec![num_complex::Complex::new(0.0, 0.0); #node_name_fft_size];
                        });

                        let in_node_names: std::collections::HashMap<
                            usize,
                            Vec<Ident>,
                            egui::ahash::RandomState,
                        > = gen_in_pins_with_ident(snarl, &node);

                        let in_data = in_node_names.get(&0).unwrap().first().unwrap().to_owned();
                        let fft_size: TokenStream = quote::quote! { #in_data.len() };

                        let out_data = gen_node_name_scratch(snarl, &node, 0);

                        code.extend(quote::quote! {
                            if #node_name_fft_size != #fft_size {
                                #node_name_fft_size = #fft_size;

                                #node_name = {
                                    let mut planner = rustfft::FftPlanner::new();
                                    let fft = planner.plan_fft_inverse(#node_name_fft_size);
                                    fft
                                };
                                #node_name_scratch_buf = vec![num_complex::Complex::new(0.0, 0.0); #node_name_fft_size];
                            }

                            let #out_data = {
                                let mut #out_data = #in_data.clone().to_vec();
                                #node_name.process_with_scratch(#out_data.as_mut_slice(), #node_name_scratch_buf.as_mut_slice());
                                #out_data.into_iter().collect::<ndarray::Array1<num_complex::Complex<f64>>>()
                            };
                        });
                    }
                },
                FlowNodes::FilterNodes(filter_node) => match filter_node {
                    FilterNodes::LifterNode(lifter_node) => {
                        let node_name = unique_node_name(snarl, node);
                        let node_name_lifter_order = Ident::new(
                            &format!("{}_lifter_order", node_name),
                            proc_macro2::Span::call_site(),
                        );

                        let in_node_names: std::collections::HashMap<
                            usize,
                            Vec<Ident>,
                            egui::ahash::RandomState,
                        > = gen_in_pins_with_ident(snarl, &node);

                        let lifter_order: TokenStream = in_node_names
                            .get(&0)
                            .map(|v| v.first().map(|v| quote::quote! { #v }))
                            .flatten()
                            .unwrap_or_else(|| {
                                let num = lifter_node.size.get();
                                quote::quote! { #num }
                            })
                            .to_owned();
                        let in_data = in_node_names.get(&1).unwrap().first().unwrap().to_owned();

                        let out_data = gen_node_name_scratch(snarl, &node, 0);

                        code.extend(quote::quote! {
                            let #out_data = {
                                let mut quefrency = #in_data.clone();
                                let index = #lifter_order;

                                for i in 0..quefrency.len() {
                                    if i < index || i >= quefrency.len() - index {
                                        quefrency[i] = 0.0;
                                    }
                                }

                                quefrency
                            };
                        });
                    }
                },
                FlowNodes::IterNodes(iter_node) => match iter_node {
                    IterNodes::EnumerateIterNode(enumerate_iter_node) => {
                        let iterated_name = unique_node_name(snarl, node);
                        let state = Ident::new(
                            &format!("{}_state", iterated_name),
                            proc_macro2::Span::call_site(),
                        );

                        outer_code.extend(quote::quote! {
                            // iterated array
                            let mut #iterated_name: ndarray::Array1<f64> = (0..10).step_by(1).map(|x| x as f64).collect();
                            // state
                            let mut #state = (0, 1, 10);
                        });

                        let in_node_names: std::collections::HashMap<
                            usize,
                            Vec<Ident>,
                            egui::ahash::RandomState,
                        > = gen_in_pins_with_ident(snarl, &node);

                        let start = in_node_names
                            .get(&0)
                            .map(|v| v.first().map(|v| quote::quote! { #v }))
                            .flatten()
                            .unwrap_or_else(|| {
                                let num = enumerate_iter_node.start.get();
                                quote::quote! { #num }
                            })
                            .to_owned();

                        let step = in_node_names
                            .get(&1)
                            .map(|v| v.first().map(|v| quote::quote! { #v }))
                            .flatten()
                            .unwrap_or_else(|| {
                                let num = enumerate_iter_node.step.get();
                                quote::quote! { #num }
                            })
                            .to_owned();

                        let end = in_node_names
                            .get(&2)
                            .map(|v| v.first().map(|v| quote::quote! { #v }))
                            .flatten()
                            .unwrap_or_else(|| {
                                let num = enumerate_iter_node.end.get();
                                quote::quote! { #num }
                            })
                            .to_owned();

                        let out_data = gen_node_name_scratch(snarl, &node, 0);

                        let gen_ident = |name: &str| {
                            Ident::new(
                                &format!("{iterated_name}_{name}"),
                                proc_macro2::Span::call_site(),
                            )
                        };
                        let start_ident = gen_ident("start");
                        let step_ident = gen_ident("step");
                        let end_ident = gen_ident("end");

                        code.extend(quote::quote! {
                            let #out_data = {
                                let (#start_ident, #step_ident, #end_ident) = (#start as usize, #step as usize, #end as usize);
                                if #state != (#start_ident, #step_ident, #end_ident) {
                                    #state = (#start_ident, #step_ident, #end_ident);
                                    #iterated_name = (#start_ident..#end_ident).step_by(#step_ident).map(|x| x as f64).collect();
                                }

                                #iterated_name.clone()
                            };
                        });
                    }
                },
                FlowNodes::LpcNodes(lpc_node) => match lpc_node {
                    LpcNodes::LpcNode(lpc_node) => {
                        let node_name = unique_node_name(snarl, node);
                        let node_name_lpc_order = Ident::new(
                            &format!("{}_lpc_order", node_name),
                            proc_macro2::Span::call_site(),
                        );

                        outer_code.extend(quote::quote! {
                            // keep the state
                            let mut #node_name_lpc_order = 10;
                        });

                        let in_node_names: std::collections::HashMap<
                            usize,
                            Vec<Ident>,
                            egui::ahash::RandomState,
                        > = gen_in_pins_with_ident(snarl, &node);

                        let lpc_order: TokenStream = in_node_names
                            .get(&0)
                            .map(|v| v.first().map(|v| quote::quote! { #v }))
                            .flatten()
                            .unwrap_or_else(|| {
                                let num = lpc_node.order.get();
                                quote::quote! { #num }
                            })
                            .to_owned();
                        let in_data = in_node_names.get(&1).unwrap().first().unwrap().to_owned();

                        let out_data = gen_node_name_scratch(snarl, &node, 0);

                        code.extend(quote::quote! {
                            if #node_name_lpc_order != #lpc_order {
                                #node_name_lpc_order = #lpc_order;
                            }

                            let #out_data = linear_predictive_coding::calc_lpc_by_levinson_durbin(
                                #in_data.view(),
                                #lpc_order,
                            );
                        });
                    }
                },
                FlowNodes::UnknownNode(_) => todo!(),
            }

            Ok(())
        }

        gen_code(&node, &snarl, &mut outer_code, &mut code).unwrap();
    }

    let code = first_code(&outer_code, &code, &hop_size);

    Ok(code)
}

// cargo test -p audio_analyzer_app --bin audio_analyzer_app -- libs::gen_code::analysis::test_analysis --exact --show-output --nocapture
#[test]
fn test_analysis() {
    // print log to stdout
    env_logger::init();

    let snarl_str = include_str!("./audio_analyzer_config.json");
    let config: crate::apps::config::Config = serde_json::from_str(&snarl_str).unwrap();
    let snarl = config.snarl;
    let code = analysis(&snarl).unwrap();

    reqwest_client::lib::rewrite_code(reqwest_client::lib::Code {
        code: code.to_string(),
    });
}

fn get_next_nodes(snarl: &Snarl<FlowNodes>, node: NodeId) -> Vec<(OutPinId, InPinId)> {
    let mut next_nodes: Vec<(OutPinId, InPinId)> = snarl
        .wires()
        .filter(|(out, _)| out.node == node)
        .collect::<Vec<_>>();

    next_nodes.sort_by(|(_, a_out), (_, b_out)| b_out.input.cmp(&a_out.input));

    next_nodes
}
