use super::OASImageViewerService;
use crate::core::ports::AppConfig;
use crate::core::{CoreError, Result};
use crate::core::use_cases::{AppState, GalleryState, ViewState};

impl OASImageViewerService {
    pub(super) fn read_state<T>(&self, f: impl FnOnce(&AppState) -> T) -> Result<T> {
        let state = self
            .state
            .lock()
            .map_err(|_| CoreError::technical("CONFIG_ERROR", "Lock poisoned".to_string()))?;
        Ok(f(&state))
    }

    pub(super) fn write_state<T>(&self, f: impl FnOnce(&mut AppState) -> T) -> Result<T> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| CoreError::technical("CONFIG_ERROR", "Lock poisoned".to_string()))?;
        Ok(f(&mut state))
    }

    pub fn new(
        view_use_case: super::ViewImageUseCase,
        navigate_use_case: super::NavigateGalleryUseCase,
        config_use_case: super::ManageConfigUseCase,
    ) -> Self {
        let config = AppConfig::default();
        let state = AppState {
            view: ViewState::default(),
            gallery: GalleryState::default(),
            config: config.clone(),
        };

        Self {
            view_use_case,
            navigate_use_case,
            config_use_case,
            state: std::sync::Mutex::new(state),
        }
    }

    pub fn initialize(&self, config: Option<AppConfig>) -> Result<()> {
        let config = if let Some(cfg) = config {
            cfg
        } else {
            self.config_use_case.load_config()?
        };

        self.write_state(|state| {
            state.config = config;
            state.gallery.layout = state.config.gallery;
        })?;

        Ok(())
    }

    pub fn get_state(&self) -> Result<AppState> {
        self.read_state(|s| s.clone())
    }

    pub fn update_state(&self, f: impl FnOnce(&mut AppState)) -> Result<()> {
        self.write_state(|state| {
            f(state);
        })
        .map(|_| ())
    }
}
