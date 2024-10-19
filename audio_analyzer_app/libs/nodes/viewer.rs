use super::NodeInfoTypesWithData;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DataPlotterNode {
    pub name: String,
    pub hold_data: Option<NodeInfoTypesWithData>,
}

pub struct DataPlotterNodeInfo;

impl super::NodeInfo for DataPlotterNodeInfo {
    fn name(&self) -> &str {
        "DataPlotterNode"
    }

    fn inputs(&self) -> usize {
        1
    }

    fn outputs(&self) -> usize {
        0
    }

    fn input_types(&self) -> Vec<super::NodeInfoTypes> {
        vec![super::NodeInfoTypes::AnyInput]
    }

    fn output_types(&self) -> Vec<super::NodeInfoTypes> {
        vec![]
    }

    fn flow_node(&self) -> super::editor::FlowNodes {
        super::editor::FlowNodes::DataPlotterNode(DataPlotterNode::default())
    }
}

impl Default for DataPlotterNode {
    fn default() -> Self {
        Self::new("DataPlotterNode".to_string())
    }
}

impl DataPlotterNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            hold_data: None,
        }
    }

    pub fn set_hold_data(&mut self, hold_data: NodeInfoTypesWithData) {
        self.hold_data = Some(hold_data);
    }

    pub fn get_hold_data(&self) -> Option<&NodeInfoTypesWithData> {
        self.hold_data.as_ref()
    }

    pub fn to_info(&self) -> DataPlotterNodeInfo {
        DataPlotterNodeInfo
    }

    pub fn show(&self, ui: &mut egui::Ui, is_new: bool) {
        ui.label(&self.name);

        if let Some(hold_data) = &self.hold_data {
            match hold_data {
                NodeInfoTypesWithData::Number(number) => {
                    ui.label(format!("Number: {}", number));
                }
                NodeInfoTypesWithData::VecF32(vec_f32) => {
                    ui.label(format!("VecF32: {:?}", vec_f32));
                }
                NodeInfoTypesWithData::Array1TupleF64F64(array1_tuple_f64_f64) => {
                    ui.label(format!("Array1TupleF64F64: {:?}", array1_tuple_f64_f64));
                }
                NodeInfoTypesWithData::Array2F64(array2_f64) => {
                    ui.label(format!("Array2F64: {:?}", array2_f64));
                }
                NodeInfoTypesWithData::Array1ComplexF64(array1_complex_f64) => {
                    ui.label(format!("Array1ComplexF64: {:?}", array1_complex_f64));
                }
            }
        }
    }
}
