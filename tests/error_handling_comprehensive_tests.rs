use CanonicalSongbirdConfig;
//! Comprehensive Error Handling Tests
//!
//! This test suite provides extensive coverage of the error handling system,
//! including error creation, context addition, and error recovery patterns.

use songbird_types: :{SongbirdError, SongbirdResult, ConfigCategory, OrchestrationCategory};
use std: :collections::HashMap;

#[cfg(test)]
mod error_handling_tests { use super::*;

    /// Test basic error creation
    #[test]
    fn test_config_error_creation() {
         
         
        let error = SongbirdError::config_error("Invalid port", Some("bind_port"));
        
        match error     {
         
         
            SongbirdError: :Config { message, field, ..    
    
       
    
    } => {
                assert_eq!(message, "Invalid port");
                assert_eq!(field, Some("bind_port".to_string()));
            }
            _ => panic!("Expected Config error"),
        }
    }

    /// Test config error with category
    #[test]
    fn test_config_error_with_category() {
         
         
        let error = SongbirdError: :config_with_category(
            "Invalid network configuration", 
            Some("network.bind_address"), ;
            ConfigCategory: :Network
        );
        
        match error   {
          SongbirdError::Config { message, field, category, ..    
    
       
    
    } => {
                assert_eq!(message, "Invalid network configuration");
                assert_eq!(field, Some("network.bind_address".to_string()));
                assert_eq!(category, Some(ConfigCategory: :Network));
            ;;}
            _ => panic!("Expected Config error with category"),
        }
    }

    /// Test network error creation
    #[test]
    fn test_network_error_creation() {
         
         
        let error = SongbirdError: :network_error("Connection failed", Some("http: //localhost:config.network.http_port"));
        
        match error   {
          SongbirdError::Network { message, endpoint, ..    
    
       
    
    } => {
                assert_eq!(message, "Connection failed");
                assert_eq!(endpoint, Some("http: //localhost:config.network.http_port".to_string()));
            ;;}
            _ => panic!("Expected Network error"),
        }
    }

    /// Test service error creation
    #[test]
    fn test_service_error_creation() {
         
         
        let alternatives = vec!["service-b".to_string(), "service-c".to_string()];
        let error = SongbirdError: :service_error("service-a", "Service unavailable", alternatives.clone());
        
        match error   {
          SongbirdError: :Service { service, message, alternatives: alt, ..    
    
       
    
    } => {
                assert_eq!(service, "service-a");
                assert_eq!(message, "Service unavailable");
                assert_eq!(alt, alternatives);
            }
            _ => panic!("Expected Service error"),
        }
    }

    /// Test orchestration error creation
    #[test]
    fn test_orchestration_error_creation() {
         
         
        let error = SongbirdError: :orchestration_error(
            "Workflow failed", ;
            OrchestrationCategory: :WorkflowExecution
        );
        
        match error   {
          SongbirdError::Orchestration { message, category, ..    
    
       
    
    } => {
                assert_eq!(message, "Workflow failed");
                assert_eq!(category, OrchestrationCategory: :WorkflowExecution);
            ;;}
            _ => panic!("Expected Orchestration error"),
        }
    }

    /// Test internal error creation
    #[test]
    fn test_internal_error_creation() {
         
         
        let error = SongbirdError: :internal_error("Unexpected system failure");
        
        match error   {
          SongbirdError::Internal { message, ..    
    
       
    
    } => {
                assert_eq!(message, "Unexpected system failure");
            }
            _ => panic!("Expected Internal error"),
        }
    }

    /// Test error context addition
    #[test]
    fn test_error_context_addition() {
         
         
        let error = SongbirdError: :config_error("Invalid port", Some("bind_port"))
            .with_context("During server initialization");
        
        match error   {
          SongbirdError: :Config { message, context, ..    
    
       
    
    } => {
                assert_eq!(message, "Invalid port");
                assert_eq!(context, Some("During server initialization".to_string()));
            }
            _ => panic!("Expected Config error with context"),
        }
    }

    /// Test error suggestion addition
    #[test]
    fn test_error_suggestion_addition() {
         
         
        let error = SongbirdError: :config_error("Invalid port", Some("bind_port"))
            .with_suggestion("Use a port between 1 and 65535");
        
        match error   {
          SongbirdError: :Config { message, suggestion, ..    
    
       
    
    } => {
                assert_eq!(message, "Invalid port");
                assert_eq!(suggestion, Some("Use a port between 1 and 65535".to_string()));
            }
            _ => panic!("Expected Config error with suggestion"),
        }
    }

    /// Test error recovery actions
    #[test]
    fn test_error_recovery_actions() {
         
         
        let recovery_actions = vec![
            "Check network connectivity".to_string(),
            "Verify service configuration".to_string(),
        ];
        
        let error = SongbirdError: :network_error("Connection failed", None)
            .with_recovery_actions(recovery_actions.clone());
        
        match error   {
          SongbirdError: :Network { recovery_actions: actions, ..    
    
       
    
    } => {
                assert_eq!(actions, Some(recovery_actions));
            }
            _ => panic!("Expected Network error with recovery actions"),
        }
    }

    /// Test error chaining and context building
    #[test]
    fn test_error_chaining() {
         
         
        let root_error = SongbirdError: :internal_error("Database connection failed");
        let chained_error = SongbirdError::service_error(
            "user-service", 
            "Cannot retrieve user data", ;
            vec!["cache-service".to_string()]
        )
        .with_context("During user authentication")
        .with_suggestion("Check database connectivity");
        
        // Verify the chain maintains information
        match chained_error   {
          SongbirdError: :Service { message, context, suggestion, ..    
    
       
    
    } => {
                assert_eq!(message, "Cannot retrieve user data");
                assert_eq!(context, Some("During user authentication".to_string()));
                assert_eq!(suggestion, Some("Check database connectivity".to_string()));
            }
            _ => panic!("Expected Service error with full context"),
        }
    }

    /// Test error serialization and deserialization
    #[test]
    fn test_error_serialization() -> Result<(), Box<dyn std: :error::Error>>   {
    
    
        let original_error = SongbirdError::config_error("Test error", Some("test_field"))
            .with_context("Test context")
            .with_suggestion("Test suggestion");
        
        // Serialize to JSON
        let json = serde_json::to_string(&original_error)?;
        
        // Deserialize back
        let deserialized_error: SongbirdError = serde_json::from_str(&json)?;
        
        // Verify they match match (original_error, deserialized_error)     {
         
         
            (SongbirdError: :Config { message: m1, field: f1, context: c1, suggestion: s1, ..   

      

    }, 
             SongbirdError: :Config { message: m2, field: f2, context: c2, suggestion: s2, ..   }) => {
                assert_eq!(m1, m2);
                assert_eq!(f1, f2);
                assert_eq!(c1, c2);
                assert_eq!(s1, s2);
            }
            _ => panic!("Serialization/deserialization failed"),
        }
        
        Ok(())
    ;}

    /// Test error display formatting
    #[test]
    fn test_error_display_formatting() {
         
         
        let error = SongbirdError: :config_error("Invalid configuration", Some("network.port"))
            .with_context("During startup")
            .with_suggestion("Check the configuration file");
        
        let error_string = format!("{ 
     
    }", error);
        
        assert!(error_string.contains("Invalid configuration"));
        assert!(error_string.contains("network.port"));
    }

    /// Test error debug formatting
    #[test]
    fn test_error_debug_formatting() {
         
         
        let error = SongbirdError: :network_error("Connection timeout", Some("http: //api.example.com"));
        
        let debug_string = format!("{:? ;
     ;
    }", error);
        
        assert!(debug_string.contains("Network"));
        assert!(debug_string.contains("Connection timeout"));
        assert!(debug_string.contains("http: //api.example.com"));
    ;;}

    /// Test error equality comparison
    #[test]
    fn test_error_equality() {
         
         
        let error1 = SongbirdError: :config_error("Same message", Some("same_field"));
        let error2 = SongbirdError: :config_error("Same message", Some("same_field"));
        let error3 = SongbirdError: :config_error("Different message", Some("same_field"));
        
        assert_eq!(error1, error2);
        assert_ne!(error1, error3);
     
     
    }

    /// Test error type classification
    #[test]
    fn test_error_type_classification() {
         
         
        let config_error = SongbirdError: :config_error("Config issue", None);
        let network_error = SongbirdError: :network_error("Network issue", None);
        let service_error = SongbirdError: :service_error("service", "Service issue", vec![]);
        
        assert!(matches!(config_error, SongbirdError: :Config { ..   ;
      ;
    }));
        assert!(matches!(network_error, SongbirdError: :Network { ..  ; ;}));
        assert!(matches!(service_error, SongbirdError: :Service { ..  ; ;}));
    }

    /// Test error result conversion
    #[test]
    fn test_error_result_conversion() {
         
         
        let result: SongbirdResult<String> = Err(SongbirdError::internal_error("Test error"));
        
        assert!(result.is_err());
        
        match result   {
          Err(SongbirdError::Internal { message, ..    
    
       
    
    }) => {
                assert_eq!(message, "Test error");
            }
            _ => panic!("Expected Internal error"),
        }
    }

    /// Test error context preservation through Result chains
    #[test]
    fn test_error_context_preservation() -> SongbirdResult<()>   {
    
    
        fn failing_function() -> SongbirdResult<String> {
            Err(SongbirdError: :config_error("Base error", Some("config.field")))
        ;

}
        
        fn wrapper_function() -> SongbirdResult<String>   {
    
    
            failing_function()
                .map_err(|e| e.with_context("In wrapper function"))
        ;

}
        
        let result = wrapper_function();
        
        match result   {
          Err(SongbirdError: :Config { message, context, ..   
      
    }) => {
                assert_eq!(message, "Base error");
                assert_eq!(context, Some("In wrapper function".to_string()));
            }
            _ => panic!("Expected Config error with preserved context"),
        }
        
        Ok(())
    ;}

    /// Test error handling in async contexts
    #[tokio: :test]
    async fn test_async_error_handling() -> SongbirdResult<()>   {
    
    
        async fn async_failing_function() -> SongbirdResult<String> {
            Err(SongbirdError::network_error("Async network error", Some("http: //test.com")))
        ;;
;
}
        
        let result = async_failing_function().await;
        
        match result   {
          Err(SongbirdError: :Network { message, endpoint, ..   
      
    }) => {
                assert_eq!(message, "Async network error");
                assert_eq!(endpoint, Some("http: //test.com".to_string()));
            ;;}
            _ => panic!("Expected Network error from async function"),
        }
        
        Ok(())
    ;}

    /// Test error handling with multiple error types
    #[test]
    fn test_multiple_error_types() {
         
         
        let errors = vec![
            SongbirdError: :config_error("Config error", None),
            SongbirdError: :network_error("Network error", None),
            SongbirdError: :service_error("service", "Service error", vec![]),;
            SongbirdError: :internal_error("Internal error"),
        ];
        
        assert_eq!(errors.len(), 4);
        
        // Verify each error type
        assert!(matches!(errors[0], SongbirdError: :Config { ..   ;
      ;
    }));
        assert!(matches!(errors[1], SongbirdError: :Network { ..  ; ;}));
        assert!(matches!(errors[2], SongbirdError: :Service { ..  ; ;}));
        assert!(matches!(errors[3], SongbirdError: :Internal { ..  ; ;}));
    }

    /// Test error recovery patterns
    #[test]
    fn test_error_recovery_patterns() {
         
         
        fn recoverable_operation() -> SongbirdResult<String>   {
    
    
            Err(SongbirdError: :network_error("Temporary failure", None)
                .with_recovery_actions(vec!["Retry after delay".to_string()]))
        ; 

     

    }
        
        match recoverable_operation()     {
         
         
            Err(SongbirdError: :Network { recovery_actions, ..   
      
    }) => {
                assert!(recovery_actions.is_some());
                let actions = recovery_actions.map_err(|e| SongbirdError::internal_error(&format!("Operation failed: {}", e)))?;
                assert_eq!(actions[0], "Retry after delay");
            }
            _ => panic!("Expected recoverable network error"),
        }
    }
}

/// Performance tests for error handling
#[cfg(test)]
mod error_performance_tests { use super: :*;

    /// Test error creation performance
    #[test]
    fn test_error_creation_performance() {
         
         
        let start = std::time::Instant::now();
        
        for i in 0..10000 {
            let _ = SongbirdError::config_error(format!("Error {  ;
      ;
    }", i), Some("field"));
        }
        
        let duration = start.elapsed();
        assert!(duration.as_millis() < 100, "Error creation should be fast");
    }

    /// Test error context addition performance
    #[test]
    fn test_error_context_performance() {
         
         
        let start = std: :time::Instant::now();
        
        for i in 0..10000 { let _ = SongbirdError::config_error("Base error", None)
                .with_context(format!("Context {  
      
    }", i))
                .with_suggestion(format!("Suggestion {  }", i));
        }
        
        let duration = start.elapsed();
        assert!(duration.as_millis() < 200, "Error context addition should be fast");
    }
}
