use stranger_strings_rs::{StrangerStrings, AnalysisOptions};

#[test]
fn test_real_model_loading() {
    let mut analyzer = StrangerStrings::new();
    
    // Test loading the actual model file
    let result = analyzer.load_model(&AnalysisOptions {
        model_path: Some("./StringModel.sng".to_string()),
        ..Default::default()
    });
    
    if result.is_ok() {
        // Model loaded successfully, verify it's configured correctly
        let (model_type, is_lowercase) = analyzer.get_model_info().unwrap();
        assert_eq!(model_type, "lowercase");
        assert!(is_lowercase);
        
        // Test some basic string analysis
        let result = analyzer.analyze_string("hello").unwrap();
        assert!(!result.original_string.is_empty());
        assert!(result.threshold < 0.0); // Should have a reasonable threshold
        
        let result = analyzer.analyze_string("test").unwrap();
        assert_eq!(result.original_string, "test");
        
        // Test that random strings get poor scores
        let result = analyzer.analyze_string("xqzfkj").unwrap();
        assert!(!result.is_valid); // Random string should be invalid
    } else {
        // Model file not found - skip this test
        println!("StringModel.sng not found, skipping real model test");
    }
}

#[test] 
fn test_basic_analysis_workflow() {
    // Test with a simple in-memory model
    let model_content = r#"# Model Type: lowercase
# Test model
[^]	h	e	100
h	e	l	200
e	l	l	150
l	l	o	180
l	o	[$]	90
[^]	t	e	50
t	e	s	80
e	s	t	70
s	t	[$]	40
"#;

    let mut analyzer = StrangerStrings::new();
    analyzer.load_model(&AnalysisOptions {
        model_content: Some(model_content.to_string()),
        ..Default::default()
    }).unwrap();

    // Test known good string
    let result = analyzer.analyze_string("hello").unwrap();
    assert_eq!(result.original_string, "hello");
    assert!(result.score > -10.0); // Should get a reasonable score
    
    // Test that strings get converted to lowercase for scoring
    let result_upper = analyzer.analyze_string("HELLO").unwrap();
    assert_eq!(result_upper.original_string, "HELLO");
    assert_eq!(result_upper.normalized_string, "hello");
    
    // Test batch analysis
    let results = analyzer.analyze_strings(&["hello".to_string(), "test".to_string()]).unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].original_string, "hello");
    assert_eq!(results[1].original_string, "test");
    
    // Test valid strings extraction
    let valid_results = analyzer.extract_valid_strings(&[
        "hello".to_string(), 
        "test".to_string(), 
        "xyz".to_string()
    ]).unwrap();
    
    // Should have some valid strings (hello and test should be valid with our test model)
    assert!(!valid_results.is_empty());
}