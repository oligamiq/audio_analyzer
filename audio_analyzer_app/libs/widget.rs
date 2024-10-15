pub trait UiWidget {
    fn is_enabled(&self, _ctx: &egui::Context) -> bool {
        true
    }

    fn name<'a>(&'a self) -> &'a str;
}

pub trait View {
    fn ui(&mut self, ui: &mut egui::Ui);
}
