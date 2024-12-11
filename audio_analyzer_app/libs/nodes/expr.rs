use crate::prelude::nodes::*;

use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use fasteval3::{Compiler, EmptyNamespace, Instruction};

use serde::Deserialize;

#[derive(serde::Serialize)]
pub struct ExprNodes {
    pub inputs_num: EditableOnText<usize>,
    pub input_var_names: Vec<String>,
    pub expr: String,
    pub outputs_num: EditableOnText<usize>,
    pub calculated: Option<NodeInfoTypesWithData>,

    #[serde(skip)]
    cb: Box<dyn Fn(&str, Vec<f64>) -> Option<f64>>,

    #[serde(skip)]
    accessor: Rc<RefCell<BTreeMap<String, f64>>>,

    #[serde(skip)]
    ret: Rc<RefCell<Ret>>,

    #[serde(skip)]
    pub slab: fasteval3::Slab,

    #[serde(skip)]
    pub compiled: Option<Instruction>,
}

impl std::fmt::Debug for ExprNodes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExprNodes")
            .field("inputs_num", &self.inputs_num)
            .field("input_var_names", &self.input_var_names)
            .field("outputs_num", &self.outputs_num)
            .field("expr", &self.expr)
            .finish()
    }
}

impl<'de> Deserialize<'de> for ExprNodes {
    fn deserialize<D>(deserializer: D) -> Result<ExprNodes, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ExprNodesHelper {
            inputs_num: EditableOnText<usize>,
            input_var_names: Vec<String>,
            expr: String,
            outputs_num: EditableOnText<usize>,
        }

        let helper = ExprNodesHelper::deserialize(deserializer)?;

        Ok(ExprNodes::new(
            helper.inputs_num.get(),
            helper.input_var_names,
            helper.expr,
            helper.outputs_num.get(),
        ))
    }
}

pub struct ExprNodeInfo;

impl Default for ExprNodeInfo {
    fn default() -> Self {
        Self
    }
}

impl NodeInfo for ExprNodeInfo {
    fn name(&self) -> &str {
        "ExprNodes"
    }

    fn inputs(&self) -> usize {
        1
    }

    fn outputs(&self) -> usize {
        1
    }

    fn input_types(&self) -> Vec<NodeInfoTypes> {
        vec![NodeInfoTypes::AnyInput]
    }

    fn output_types(&self) -> Vec<NodeInfoTypes> {
        vec![NodeInfoTypes::AnyOutput]
    }

    fn flow_node(&self) -> super::editor::FlowNodes {
        super::editor::FlowNodes::ExprNode(Default::default())
    }
}

impl Default for ExprNodes {
    fn default() -> Self {
        Self::new(1, vec!["x".to_string()], "x * 3".to_string(), 1)
    }
}

#[derive(Debug, Clone)]
enum Ret {
    Tuple(Vec<f64>),
    Complex(f64, f64),
}

impl Ret {
    fn new() -> Self {
        Ret::Tuple(Vec::new())
    }
}

impl ExprNodes {
    fn new(
        inputs_num: usize,
        input_var_names: Vec<String>,
        expr: String,
        outputs_num: usize,
    ) -> Self {
        let (cb, access_vars, ret_values) = Self::create_cb();

        let mut sl = Self {
            inputs_num: EditableOnText::new(inputs_num),
            input_var_names,
            expr,
            outputs_num: EditableOnText::new(outputs_num),
            calculated: None,
            cb,
            accessor: access_vars,
            ret: ret_values,
            slab: fasteval3::Slab::new(),
            compiled: None,
        };

        sl.update();

        sl
    }

    fn create_cb() -> (
        Box<dyn Fn(&str, Vec<f64>) -> Option<f64>>,
        Rc<RefCell<BTreeMap<String, f64>>>,
        Rc<RefCell<Ret>>,
    ) {
        let access_vars: Rc<RefCell<BTreeMap<String, f64>>> =
            Rc::new(RefCell::new(BTreeMap::new()));

        let ret_values = Rc::new(RefCell::new(Ret::new()));

        let ret_values_clone = ret_values.clone();

        let access_vars_clone = access_vars.clone();

        let cb = Box::new(move |name: &str, args: Vec<f64>| -> Option<f64> {
            match name {
                "tuple" => {
                    let mut ret_values = ret_values_clone.borrow_mut();
                    let ret_values = match &mut *ret_values {
                        Ret::Tuple(ret_values) => ret_values,
                        _ => unreachable!(),
                    };
                    for arg in args {
                        ret_values.push(arg);
                    }
                    return Some(0.0);
                }
                "sqrt" => {
                    if args.len() == 1 {
                        return Some(args[0].sqrt());
                    }
                }
                "log_10" => {
                    if args.len() == 1 {
                        return Some(args[0].log10());
                    }
                }
                "complex" => {
                    if args.len() == 2 {
                        let mut ret_values = ret_values_clone.borrow_mut();
                        *ret_values = Ret::Complex(args[0], args[1]);
                        return Some(0.0);
                    }
                }
                _ => {}
            }

            let access_vars = access_vars_clone.borrow();

            if let Some(var) = access_vars.get(name) {
                return Some(*var);
            }
            None
        });

        (cb, access_vars, ret_values)
    }

    fn eval(&mut self, inputs: Vec<f64>) -> Option<Ret> {
        use fasteval3::Evaler;

        let Self {
            cb,
            accessor,
            ret,
            compiled,
            inputs_num,
            input_var_names,
            outputs_num,
            slab,
            ..
        } = self;

        if inputs.is_empty() || inputs.len() != inputs_num.get() {
            return None;
        }

        let mut access_vars = accessor.borrow_mut();
        access_vars.clear();

        for (name, value) in input_var_names.iter().zip(inputs.iter()) {
            access_vars.insert(name.clone(), *value);
        }

        std::mem::drop(access_vars);

        if let Some(compiled) = compiled {
            let mut cb = cb.as_ref();

            let mut ret_values = ret.borrow_mut();
            *ret_values = Ret::new();
            std::mem::drop(ret_values);

            let result = if let fasteval3::IConst(c) = compiled {
                *c
            } else {
                match compiled.eval(slab, &mut cb) {
                    Ok(result) => result,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        return None;
                    }
                }
            };

            let ret = ret.borrow();

            match &*ret {
                Ret::Tuple(ret) => {
                    if ret.len() == 0 {
                        if outputs_num.get() == 1 {
                            return Some(Ret::Tuple(vec![result]));
                        } else {
                            log::error!("Outputs number is not matched");

                            return None;
                        }
                    } else {
                        if ret.len() == outputs_num.get() {
                            let ret = ret.clone();
                            return Some(Ret::Tuple(ret));
                        } else {
                            log::error!("Outputs number is not matched");

                            return None;
                        }
                    }
                }
                Ret::Complex(re, im) => {
                    if outputs_num.get() == 2 {
                        return Some(Ret::Complex(*re, *im));
                    } else {
                        log::error!("Outputs number is not matched");

                        return None;
                    }
                }
            }
        }

        None
    }

    fn show_and_calc(
        &mut self,
        ctx: &FlowNodesViewerCtx,
        ui: &mut egui::Ui,
        data: Option<NodeInfoTypesWithData>,
    ) -> PinInfo {
        ui.label("outputs_num");

        if egui::TextEdit::singleline(&mut self.outputs_num)
            .clip_text(false)
            .desired_width(0.0)
            .margin(ui.spacing().item_spacing)
            .show(ui)
            .response
            .lost_focus()
        {
            self.outputs_num.fmt();
        }

        if !ctx.running {
            return CustomPinInfo::none_status();
        }

        if let Some(data) = &data {
            match &data {
                NodeInfoTypesWithData::Number(num) => {
                    self.inputs_num.set(1);
                    self.input_var_names = vec!["x".to_string()];

                    match self.outputs_num.get() {
                        1 => {
                            if let Some(y) = self.eval(vec![*num]) {
                                let y = match y {
                                    Ret::Tuple(y) => y[0],
                                    Ret::Complex(re, _) => re,
                                };
                                self.calculated = Some(NodeInfoTypesWithData::Number(y));

                                return CustomPinInfo::ok_status();
                            }
                        }
                        2 => {
                            let y = self.eval(vec![*num]);

                            if let Some(y) = y {
                                match y {
                                    Ret::Complex(re, im) => {
                                        self.calculated =
                                            Some(NodeInfoTypesWithData::Array1ComplexF64(
                                                ndarray::Array1::from(vec![
                                                    num_complex::Complex::new(re, im),
                                                ]),
                                            ));

                                        return PinInfo::circle()
                                            .with_fill(egui::Color32::from_rgb(0, 255, 0));
                                    }
                                    Ret::Tuple(y) => {
                                        self.calculated =
                                            Some(NodeInfoTypesWithData::Array1TupleF64F64(
                                                ndarray::Array1::from(vec![(
                                                    y[0] as f64,
                                                    y[1] as f64,
                                                )]),
                                            ));

                                        return PinInfo::circle()
                                            .with_fill(egui::Color32::from_rgb(0, 255, 0));
                                    }
                                }
                            }
                        }
                        _ => {}
                    }

                    return CustomPinInfo::ok_status();
                }
                NodeInfoTypesWithData::Array1F64(_)
                | NodeInfoTypesWithData::Array1TupleF64F64(_)
                | NodeInfoTypesWithData::Array1ComplexF64(_) => {
                    log::info!("Array1F64");

                    let array = match data {
                        NodeInfoTypesWithData::Array1F64(array) => {
                            self.inputs_num.set(2);
                            self.input_var_names = vec!["x".to_string(), "y".to_string()];

                            array
                                .iter()
                                .map(|x| self.eval(vec![*x as f64]))
                                .collect::<Option<Array1<_>>>()
                        }
                        NodeInfoTypesWithData::Array1TupleF64F64(array) => {
                            self.inputs_num.set(2);
                            self.input_var_names = vec!["x".to_string(), "y".to_string()];

                            array
                                .iter()
                                .map(|x| self.eval(vec![x.0 as f64, x.1 as f64]))
                                .collect::<Option<Array1<_>>>()
                        }
                        NodeInfoTypesWithData::Array1ComplexF64(array) => {
                            self.inputs_num.set(2);
                            self.input_var_names = vec!["x".to_string(), "y".to_string()];

                            array
                                .iter()
                                .map(|x| self.eval(vec![x.re, x.im]))
                                .collect::<Option<Array1<_>>>()
                        }
                        _ => unreachable!(),
                    };

                    match self.outputs_num.get() {
                        1 => {
                            if let Some(y) = array {
                                self.calculated = Some(NodeInfoTypesWithData::Array1F64(
                                    y.into_iter()
                                        .map(|y| match y {
                                            Ret::Tuple(y) => y[0],
                                            Ret::Complex(re, _) => re,
                                        })
                                        .collect::<Array1<_>>(),
                                ));

                                return CustomPinInfo::ok_status();
                            }
                        }
                        2 => {
                            if let Some(ref ty) = array.as_ref().map(|y| y.get(0)).flatten() {
                                match ty {
                                    Ret::Tuple(..) => {
                                        self.calculated =
                                            Some(NodeInfoTypesWithData::Array1TupleF64F64(
                                                array
                                                    .unwrap()
                                                    .into_iter()
                                                    .map(|y| match y {
                                                        Ret::Tuple(y) => (y[0] as f64, y[1] as f64),
                                                        _ => unreachable!(),
                                                    })
                                                    .collect::<Array1<_>>(),
                                            ));
                                    }
                                    Ret::Complex(..) => {
                                        self.calculated =
                                            Some(NodeInfoTypesWithData::Array1ComplexF64(
                                                array
                                                    .unwrap()
                                                    .into_iter()
                                                    .map(|y| match y {
                                                        Ret::Complex(re, im) => {
                                                            num_complex::Complex::new(re, im)
                                                        }
                                                        _ => unreachable!(),
                                                    })
                                                    .collect::<Array1<_>>(),
                                            ));
                                    }
                                }
                            }

                            return CustomPinInfo::ok_status();
                        }
                        _ => unreachable!(),
                    }
                }
                NodeInfoTypesWithData::Array2F64(_) => unimplemented!(),
            };
        }

        self.calculated = None;

        CustomPinInfo::none_status()
    }
}

impl FlowNodesViewerTrait for ExprNodes {
    fn show_input(
        &self,
        ctx: &FlowNodesViewerCtx,
        pin: &egui_snarl::InPin,
        _: &mut egui::Ui,
        _: f32,
        snarl: &egui_snarl::Snarl<FlowNodes>,
    ) -> Box<dyn Fn(&mut Snarl<FlowNodes>, &mut egui::Ui) -> PinInfo> {
        assert!(pin.id.input == 0);

        let pin_id = pin.id;

        let data = pin
            .remotes
            .get(0)
            .map(|out_pin| snarl[out_pin.node].to_node_info_types_with_data(out_pin.output))
            .flatten();

        let ctx = ctx.clone();

        return Box::new(move |snarl: &mut Snarl<FlowNodes>, ui: &mut egui::Ui| {
            if let FlowNodes::ExprNode(node) = &mut snarl[pin_id.node] {
                config_ui!(node, ui, expr);

                return node.show_and_calc(&ctx, ui, data.clone());
            }

            CustomPinInfo::none_status()
        });
    }
}

impl GraphNode for ExprNodes {
    type NodeInfoType = ExprNodeInfo;

    fn to_info(&self) -> Self::NodeInfoType {
        ExprNodeInfo
    }

    fn update(&mut self) {
        let Self { expr, slab, .. } = self;

        let parser = fasteval3::Parser::new();
        if let Ok(parsed) = parser.parse(expr, &mut slab.ps) {
            let compiled =
                parsed
                    .from(&slab.ps)
                    .compile(&slab.ps, &mut slab.cs, &mut EmptyNamespace);

            self.compiled = Some(compiled);
        }
    }
}
