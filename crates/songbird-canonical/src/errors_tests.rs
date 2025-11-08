//! Unit tests for canonical error types

#[cfg(test)]
mod tests {
    #![allow(clippy::uninlined_format_args)]
    #![allow(clippy::float_cmp)]
    #![allow(clippy::useless_vec)]
    #![allow(clippy::unreadable_literal)]

    use crate::errors::*;
    use songbird_types::SongbirdResult;

    #[test]
    fn test_error_context_creation() {
        let context = ErrorContext::new("Test error", "Test context");

        assert_eq!(context.message(), "Test error");
        assert_eq!(context.context(), "Test context");
        assert!(context.suggestions().is_empty());
    }

    #[test]
    fn test_error_context_with_suggestion() {
        let context = ErrorContext::new("Error", "Context").with_suggestion("Try this");

        assert_eq!(context.suggestions().len(), 1);
        assert_eq!(context.suggestions()[0], "Try this");
    }

    #[test]
    fn test_error_context_with_multiple_suggestions() {
        let suggestions = vec!["Suggestion 1", "Suggestion 2", "Suggestion 3"];
        let context = ErrorContext::new("Error", "Context").with_suggestions(suggestions.clone());

        assert_eq!(context.suggestions().len(), 3);
        for (i, suggestion) in context.suggestions().iter().enumerate() {
            assert_eq!(suggestion, suggestions[i]);
        }
    }

    #[test]
    fn test_error_context_clone() {
        let context1 = ErrorContext::new("Error", "Context").with_suggestion("Fix it");
        let context2 = context1.clone();

        assert_eq!(context1.message(), context2.message());
        assert_eq!(context1.context(), context2.context());
        assert_eq!(context1.suggestions().len(), context2.suggestions().len());
    }

    #[test]
    fn test_error_context_display() -> SongbirdResult<()> {
        let context = ErrorContext::new("Test error", "In module X").with_suggestion("Check logs");

        let display = format!("{context}");
        assert!(display.contains("Test error"));
        assert!(display.contains("In module X"));
        assert!(display.contains("Check logs"));
        Ok(())
    }

    #[test]
    fn test_error_context_display_no_suggestions() -> SongbirdResult<()> {
        let context = ErrorContext::new("Simple error", "Simple context");

        let display = format!("{context}");
        assert!(display.contains("Simple error"));
        assert!(display.contains("Simple context"));
        Ok(())
    }

    #[test]
    fn test_error_context_debug() -> SongbirdResult<()> {
        let context = ErrorContext::new("Debug test", "Debug context");

        let debug_str = format!("{context:?}");
        assert!(debug_str.contains("ErrorContext"));
        assert!(debug_str.contains("Debug test"));
        Ok(())
    }

    #[test]
    fn test_error_context_empty_strings() {
        let context = ErrorContext::new("", "");

        assert!(context.message().is_empty());
        assert!(context.context().is_empty());
    }

    #[test]
    fn test_error_context_unicode() {
        let context = ErrorContext::new("エラー", "コンテキスト");

        assert_eq!(context.message(), "エラー");
        assert_eq!(context.context(), "コンテキスト");
    }

    #[test]
    fn test_error_context_long_message() {
        let long_message = "x".repeat(1000);
        let context = ErrorContext::new(long_message, "context");

        assert_eq!(context.message().len(), 1000);
    }

    #[test]
    fn test_error_context_many_suggestions() {
        let mut context = ErrorContext::new("Error", "Context");
        for i in 0..50 {
            context = context.with_suggestion(format!("Suggestion {i}"));
        }

        assert_eq!(context.suggestions().len(), 50);
    }

    #[test]
    fn test_error_context_special_characters() {
        let context = ErrorContext::new("Error: !@#$%^&*()", "Context with \n newlines \t tabs");

        assert!(context.message().contains("!@#$%"));
        assert!(context.context().contains('\n'));
    }

    #[test]
    fn test_error_context_builder_pattern() {
        let context = ErrorContext::new("Network error", "Connection failed")
            .with_suggestion("Check network")
            .with_suggestion("Retry connection")
            .with_suggestion("Contact admin");

        assert_eq!(context.suggestions().len(), 3);
    }

    #[test]
    fn test_error_context_message_getter() {
        let context = ErrorContext::new("Message test", "Context");
        let message = context.message();

        assert_eq!(message, "Message test");
    }

    #[test]
    fn test_error_context_context_getter() {
        let context = ErrorContext::new("Error", "Context test");
        let ctx = context.context();

        assert_eq!(ctx, "Context test");
    }

    #[test]
    fn test_error_context_suggestions_getter() {
        let context = ErrorContext::new("Error", "Context").with_suggestions(vec!["A", "B", "C"]);
        let suggestions = context.suggestions();

        assert_eq!(suggestions.len(), 3);
        assert_eq!(suggestions[0], "A");
    }

    #[test]
    fn test_success_result() {
        let value = 42;
        let result = success_result(value);

        assert_eq!(result, 42);
    }

    #[test]
    fn test_success_result_string() {
        let value = "test".to_string();
        let result = success_result(value.clone());

        assert_eq!(result, value);
    }

    #[test]
    fn test_success_result_complex_type() {
        let value = vec![1, 2, 3];
        let result = success_result(value.clone());

        assert_eq!(result, value);
    }

    #[test]
    fn test_unit_success() {
        let result = unit_success();
        assert!(result.is_ok());
    }

    #[test]
    fn test_unit_success_multiple_calls() {
        for _ in 0..10 {
            let result = unit_success();
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_error_context_combined_operations() {
        let context = ErrorContext::new("Combined test", "Multiple operations")
            .with_suggestion("First")
            .with_suggestions(vec!["Second", "Third"])
            .with_suggestion("Fourth");

        assert_eq!(context.suggestions().len(), 4);
        assert_eq!(context.message(), "Combined test");
        assert_eq!(context.context(), "Multiple operations");
    }
}
