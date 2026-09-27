use pyo3::prelude::*;

#[test]
fn test_adapters_module_loading() {
    pyo3::prepare_freethreaded_python();

    Python::with_gil(|py| {
        let module = PyModule::new(py, "test_module").expect("Failed to create PyModule");
        contextra_adapters::_contextra_adapters(py, &module)
            .expect("Failed to initialize _contextra_adapters module");

        assert!(module.hasattr("LangChainChatMessageHistory").unwrap_or(false));
        assert!(module.hasattr("LangGraphStoreAdapter").unwrap_or(false));
        assert!(module.hasattr("LlamaIndexStoreAdapter").unwrap_or(false));
    });
}
