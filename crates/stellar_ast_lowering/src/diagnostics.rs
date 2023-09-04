use stellar_diagnostics::define_diagnostics;
use stellar_filesystem::location::Location;

define_diagnostics! {
    diagnostic(warning) UnnecessaryGroupedPattern(
        self,
        location: Location
    ) {
        code { "W000" }
        message { "unnecessary grouped pattern" }
        files_involved { self.location.filepath }
        labels {
            primary self.location.start_byte_location() => {""},
            secondary self.location.end_byte_location() => {
                "consider removing these parentheses"
            }
        }
        notes {}
    }

    diagnostic(warning) UnnecessaryParenthesizedExpression(
        self,
        location: Location
    ) {
        code { "W001" }
        message { "unnecessary parenthesized expression" }
        files_involved { self.location.filepath }
        labels {
            primary self.location.start_byte_location() => {""},
            secondary self.location.end_byte_location() => {
                "consider removing these parentheses"
            }
        }
        notes {}
    }

    diagnostic(warning) UnnecessaryParenthesizedType(
        self,
        location: Location
    ) {
        code { "W002" }
        message { "unnecessary parenthesized type" }
        files_involved { self.location.filepath }
        labels {
            primary self.location.start_byte_location() => {""},
            secondary self.location.end_byte_location() => {
                "consider removing these parentheses"
            }
        }
        notes {}
    }
}
