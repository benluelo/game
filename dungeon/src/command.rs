// pub struct Command {
//     name: Option<String>,
//     description: Option<String>,
//     usages: Option<Vec<String>>,
//     arguments: Option<Vec<CommandArgument>>,
//     options: Option<Vec<CommandOption>>,
//     subcommands: Option<Vec<Command>>,
// }

// impl Command {
//     pub fn builder() -> CommandBuilder {
//         Default::default()
//     }
// }

// pub struct CommandArgument {}
// pub struct CommandOption {}

// pub struct CommandBuilder {
//     name: Option<String>,
//     description: Option<String>,
//     usages: Option<Vec<String>>,
//     arguments: Option<Vec<CommandArgument>>,
//     options: Option<Vec<CommandOption>>,
//     subcommands: Option<Vec<Command>>,
// }

// impl Default for CommandBuilder {
//     fn default() -> Self {
//         CommandBuilder {
//             ..Default::default()
//         }
//     }
// }

// impl CommandBuilder {
//     pub fn name(mut self, name: String) -> Self {
//         Self {
//             name: Some(name),
//             ..self
//         }
//     }
//     pub fn description(mut self, description: String) -> Self {
//         Self {
//             description: Some(description),
//             ..self
//         }
//     }
//     pub fn usages(mut self, usages: Vec<String>) -> Self {
//         Self {
//             usages: Some(usages),
//             ..self
//         }
//     }
//     pub fn arguments(mut self, arguments: Vec<CommandArgument>) -> Self {
//         Self {
//             arguments: Some(arguments),
//             ..self
//         }
//     }
//     pub fn options(mut self, options: Vec<CommandOption>) -> Self {
//         Self {
//             options: Some(options),
//             ..self
//         }
//     }
//     pub fn subcommands(mut self, subcommands: Vec<Command>) -> Self {
//         Self {
//             subcommands: Some(subcommands),
//             ..self
//         }
//     }

//     pub fn build(self) -> Command {
//         Command {
//             name: self.name,
//             description: self.description,
//             usages: self.usages,
//             arguments: self.arguments,
//             options: self.options,
//             subcommands: self.subcommands,
//         }
//     }
// }

// fn main() {
//     let command = Command::builder()
//         .name("hello".into())
//         .description("does stuff".into())
//         .arguments(vec![
//             CommandArgument {},
//             CommandArgument {},
//             CommandArgument {},
//         ])
//         .build();
// }
