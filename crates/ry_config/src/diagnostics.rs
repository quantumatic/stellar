use ry_filesystem::span::Span;

pub enum ConfigFileDiagnostic {
    InvalidFormat {
        message: String,
        span: Span,
    },
}
