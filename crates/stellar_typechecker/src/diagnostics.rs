use stellar_diagnostics::define_diagnostics;
use stellar_filesystem::location::Location;

define_diagnostics! {
    diagnostic(error) DuplicateModuleItem(
        self,
        name: String,
        first_definition_location: Location,
        second_definition_location: Location
    ) {
        code { "E005" }
        message { format!("duplicate definition of the module item `{}`", self.name) }
        labels {
            primary self.first_definition_location => {
                format!("first definition of `{}`", self.name)
            },
            secondary self.second_definition_location => {
                format!("second, conflicting definition of `{}`", self.name)
            }
        }
        notes {}
    }

    diagnostic(error) DuplicateEnumItem(
        self,
        enum_name: String,
        item_name: String,
        first_definition_location: Location,
        second_definition_location: Location
    ) {
        code { "E006" }
        message { format!("duplicate definition of the enum item `{}` in `{}`", self.item_name, self.enum_name) }
        labels {
            primary self.first_definition_location => {
                format!("first definition of `{}`", self.item_name)
            },
            secondary self.second_definition_location => {
                format!("second, conflicting definition of `{}`", self.item_name)
            }
        }
        notes {}
    }
}
