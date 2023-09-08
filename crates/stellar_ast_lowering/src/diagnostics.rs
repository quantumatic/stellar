use stellar_diagnostics::define_diagnostics;
use stellar_filesystem::location::Location;

define_diagnostics! {
    diagnostic(warning) UnnecessaryGroupedPattern(
        self,
        location: Location
    ) {
        code { "W000" }
        message { "unnecessary grouped pattern" }
        labels {
            primary { self.location.start_byte_location() }
            primary { self.location.end_byte_location() => "help: remove these parentheses" }
        }
    }

    diagnostic(warning) UnnecessaryParenthesizedExpression(
        self,
        location: Location
    ) {
        code { "W001" }
        message { "unnecessary parenthesized expression" }
        labels {
            primary { self.location.start_byte_location() }
            primary { self.location.end_byte_location() => "help: remove these parentheses" }
        }
    }

    diagnostic(warning) UnnecessaryParenthesizedType(
        self,
        location: Location
    ) {
        code { "W002" }
        message { "unnecessary parenthesized type" }
        labels {
            primary { self.location.start_byte_location() }
            primary { self.location.end_byte_location() => "help: remove these parentheses" }
        }
    }
}
