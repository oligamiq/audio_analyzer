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
    ret: Rc<RefCell<Vec<f64>>>,

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

impl ExprNodes {
    pub fn new(
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

    pub fn to_info(&self) -> ExprNodeInfo {
        ExprNodeInfo
    }

    pub fn create_cb() -> (
        Box<dyn Fn(&str, Vec<f64>) -> Option<f64>>,
        Rc<RefCell<BTreeMap<String, f64>>>,
        Rc<RefCell<Vec<f64>>>,
    ) {
        let access_vars: Rc<RefCell<BTreeMap<String, f64>>> =
            Rc::new(RefCell::new(BTreeMap::new()));

        let ret_values = Rc::new(RefCell::new(Vec::new()));

        let ret_values_clone = ret_values.clone();

        let access_vars_clone = access_vars.clone();

        let cb = Box::new(move |name: &str, args: Vec<f64>| -> Option<f64> {
            match name {
                "tuple" => {
                    let mut ret_values = ret_values_clone.borrow_mut();
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

    pub fn update(&mut self) {
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

    pub fn eval(&mut self, inputs: Vec<f64>) -> Option<Vec<f64>> {
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

        assert!(inputs.len() == inputs_num.get());

        let mut access_vars = accessor.borrow_mut();
        access_vars.clear();

        for (name, value) in input_var_names.iter().zip(inputs.iter()) {
            access_vars.insert(name.clone(), *value);
        }

        std::mem::drop(access_vars);

        if let Some(compiled) = compiled {
            let mut cb = cb.as_ref();

            let mut ret_values = ret.borrow_mut();
            ret_values.clear();
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
            if ret.len() == 0 {
                if outputs_num.get() == 1 {
                    return Some(vec![result]);
                } else {
                    log::error!("Outputs number is not matched");

                    return None;
                }
            } else {
                if ret.len() == outputs_num.get() {
                    let ret = ret.clone();
                    return Some(ret);
                } else {
                    log::error!("Outputs number is not matched");

                    return None;
                }
            }
        }

        None
    }

    pub fn show_and_calc(
        &mut self,
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

        if let Some(data) = &data {
            match data {
                NodeInfoTypesWithData::Number(num) => {
                    self.inputs_num.set(1);
                    self.input_var_names = vec!["x".to_string()];

                    // let y = self.eval(vec![*num]);

                    // if let Some(y) = y {
                    //     self.calculated = Some(NodeInfoTypesWithData::Number(y[0] as f64));

                    //     return PinInfo::circle().with_fill(egui::Color32::from_rgb(0, 255, 0));
                    // }

                    match self.outputs_num.get() {
                        1 => {
                            let y = self.eval(vec![*num]);

                            if let Some(y) = y {
                                self.calculated = Some(NodeInfoTypesWithData::Number(y[0] as f64));

                                return PinInfo::circle()
                                    .with_fill(egui::Color32::from_rgb(0, 255, 0));
                            }
                        }
                        2 => {
                            let y = self.eval(vec![*num]);

                            if let Some(y) = y {
                                self.calculated = Some(NodeInfoTypesWithData::Array1TupleF64F64(
                                    ndarray::Array1::from(vec![(y[0] as f64, y[1] as f64)]),
                                ));

                                return PinInfo::circle()
                                    .with_fill(egui::Color32::from_rgb(0, 255, 0));
                            }
                        }
                        _ => {}
                    }

                    return PinInfo::circle().with_fill(egui::Color32::from_rgb(0, 255, 0));
                }
                NodeInfoTypesWithData::Array1ComplexF64(array) => {
                    self.inputs_num.set(2);
                    self.input_var_names = vec!["x".to_string(), "y".to_string()];

                    match self.outputs_num.get() {
                        1 => {
                            let y = array
                                .iter()
                                .map(|x| self.eval(vec![x.re, x.im]).map(|y| y[0] as f32))
                                .collect::<Option<Vec<_>>>();

                            if let Some(y) = y {
                                self.calculated = Some(NodeInfoTypesWithData::VecF32(y));

                                return PinInfo::circle()
                                    .with_fill(egui::Color32::from_rgb(0, 255, 0));
                            }
                        }
                        2 => {
                            let y = array
                                .iter()
                                .map(|x| {
                                    self.eval(vec![x.re, x.im])
                                        .map(|y| (y[0] as f64, y[1] as f64))
                                })
                                .collect::<Option<ndarray::Array1<_>>>();

                            if let Some(y) = y {
                                self.calculated = Some(NodeInfoTypesWithData::Array1TupleF64F64(y));

                                return PinInfo::circle()
                                    .with_fill(egui::Color32::from_rgb(0, 255, 0));
                            }
                        }
                        _ => {}
                    }
                }
                NodeInfoTypesWithData::VecF32(array) => {
                    self.inputs_num.set(1);
                    self.input_var_names = vec!["x".to_string()];

                    match self.outputs_num.get() {
                        1 => {
                            let y = array
                                .iter()
                                .map(|x| self.eval(vec![*x as f64]).map(|y| y[0] as f32))
                                .collect::<Option<Vec<_>>>();

                            if let Some(y) = y {
                                self.calculated = Some(NodeInfoTypesWithData::VecF32(y));

                                return PinInfo::circle()
                                    .with_fill(egui::Color32::from_rgb(0, 255, 0));
                            }
                        }
                        2 => {
                            let y = array
                                .iter()
                                .map(|x| {
                                    self.eval(vec![*x as f64])
                                        .map(|y| (y[0] as f64, y[1] as f64))
                                })
                                .collect::<Option<ndarray::Array1<_>>>();

                            if let Some(y) = y {
                                self.calculated = Some(NodeInfoTypesWithData::Array1TupleF64F64(y));

                                return PinInfo::circle()
                                    .with_fill(egui::Color32::from_rgb(0, 255, 0));
                            }
                        }
                        _ => {}
                    }
                }
                NodeInfoTypesWithData::Array1F64(array) => {
                    self.inputs_num.set(1);
                    self.input_var_names = vec!["x".to_string()];

                    match self.outputs_num.get() {
                        1 => {
                            let y = array
                                .iter()
                                .map(|x| self.eval(vec![*x as f64]).map(|y| y[0] as f32))
                                .collect::<Option<Vec<_>>>();

                            if let Some(y) = y {
                                self.calculated = Some(NodeInfoTypesWithData::VecF32(y));

                                return PinInfo::circle()
                                    .with_fill(egui::Color32::from_rgb(0, 255, 0));
                            }
                        }
                        2 => {
                            let y = array
                                .iter()
                                .map(|x| {
                                    self.eval(vec![*x as f64])
                                        .map(|y| (y[0] as f64, y[1] as f64))
                                })
                                .collect::<Option<ndarray::Array1<_>>>();

                            if let Some(y) = y {
                                self.calculated = Some(NodeInfoTypesWithData::Array1TupleF64F64(y));

                                return PinInfo::circle()
                                    .with_fill(egui::Color32::from_rgb(0, 255, 0));
                            }
                        }
                        _ => {}
                    }
                }
                NodeInfoTypesWithData::Array1TupleF64F64(array) => {
                    self.inputs_num.set(2);
                    self.input_var_names = vec!["x".to_string(), "y".to_string()];

                    match self.outputs_num.get() {
                        1 => {
                            let y = array
                                .iter()
                                .map(|x| self.eval(vec![x.0, x.1]).map(|y| y[0] as f32))
                                .collect::<Option<Vec<_>>>();

                            if let Some(y) = y {
                                self.calculated = Some(NodeInfoTypesWithData::VecF32(y));

                                return PinInfo::circle()
                                    .with_fill(egui::Color32::from_rgb(0, 255, 0));
                            }
                        }
                        2 => {
                            let y = array
                                .iter()
                                .map(|x| {
                                    self.eval(vec![x.0, x.1])
                                        .map(|y| (y[0] as f64, y[1] as f64))
                                })
                                .collect::<Option<ndarray::Array1<_>>>();

                            if let Some(y) = y {
                                self.calculated = Some(NodeInfoTypesWithData::Array1TupleF64F64(y));

                                return PinInfo::circle()
                                    .with_fill(egui::Color32::from_rgb(0, 255, 0));
                            }
                        }
                        _ => {}
                    }
                }
                NodeInfoTypesWithData::Array2F64(_array) => {
                    unimplemented!()
                }
            }
        }

        self.calculated = None;

        PinInfo::circle().with_fill(egui::Color32::from_rgb(0, 0, 0))
    }
}

impl FlowNodesViewerTrait for ExprNodes {
    fn show_input(
        &self,
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

        return Box::new(move |snarl: &mut Snarl<FlowNodes>, ui: &mut egui::Ui| {
            if let FlowNodes::ExprNode(node) = &mut snarl[pin_id.node] {
                config_ui!(node, ui, expr);

                return node.show_and_calc(ui, data.clone());
            }

            PinInfo::circle().with_fill(egui::Color32::from_rgb(0, 0, 0))
        });
    }
}
