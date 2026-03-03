// use std::collections::HashMap;
// use regex::Regex;

// struct Publisher<> {
//     map:HashMap<String,Regex>
// }


// #[derive(Default)]
// struct Editor {
//     publisher: Publisher,
//     file_path: String,
// }

// impl Context {
//     pub fn events(&mut self) -> &mut Publisher {
//         &mut self.publisher
//     }

//     pub fn load(&mut self, path: String) {
//         self.file_path = path.clone();
//         self.publisher.notify(Event::Load, path);
//     }

//     pub fn save(&self) {
//         self.publisher.notify(Event::Save, self.file_path.clone());
//     }
// }
