use super::types::{EguiApp, SlideshowEndBehavior};
use crate::core::domain::{NavigationDirection, ViewMode};
use egui::Context;
use std::time::{Duration, Instant};

impl EguiApp {
    pub(super) fn tick_slideshow(&mut self, ctx: &Context) {
        if !self.slideshow.playing {
            return;
        }
        if self.service.get_view_mode().ok() != Some(ViewMode::Viewer) {
            return;
        }

        let is_focused = ctx.input(|i| i.viewport().focused.unwrap_or(true));
        if !is_focused {
            self.slideshow.paused_by_background = true;
            ctx.request_repaint_after(Duration::from_millis(200));
            return;
        }
        if self.slideshow.paused_by_background {
            self.slideshow.paused_by_background = false;
            self.slideshow.last_advanced_at = Instant::now();
            ctx.request_repaint_after(Duration::from_secs(self.slideshow.interval_seconds));
            return;
        }

        let interval = Duration::from_secs(self.slideshow.interval_seconds);
        let elapsed = Instant::now().saturating_duration_since(self.slideshow.last_advanced_at);
        if elapsed < interval {
            ctx.request_repaint_after(interval.saturating_sub(elapsed));
            return;
        }
        self.slideshow.last_advanced_at = Instant::now();
        self.advance_slideshow(ctx);
        ctx.request_repaint_after(interval);
    }

    pub(crate) fn toggle_slideshow(&mut self) {
        if self.slideshow.playing {
            self.pause_slideshow();
        } else {
            self.start_slideshow();
        }
    }

    pub(crate) fn start_slideshow(&mut self) {
        self.slideshow.playing = true;
        self.slideshow.paused_by_background = false;
        self.slideshow.last_advanced_at = Instant::now();
    }

    pub(crate) fn pause_slideshow(&mut self) {
        self.slideshow.playing = false;
        self.slideshow.paused_by_background = false;
    }

    pub(crate) fn set_slideshow_interval(&mut self, interval_seconds: u64) {
        self.slideshow.interval_seconds = normalized_interval(interval_seconds);
        self.slideshow.last_advanced_at = Instant::now();
    }

    pub(crate) fn set_slideshow_end_behavior(&mut self, behavior: SlideshowEndBehavior) {
        self.slideshow.end_behavior = behavior;
    }

    pub(crate) fn bump_slideshow_timer(&mut self) {
        self.slideshow.last_advanced_at = Instant::now();
    }

    fn advance_slideshow(&mut self, ctx: &Context) {
        let next_index = self.service.navigate_gallery(NavigationDirection::Next).ok().flatten();
        if let Some(index) = next_index {
            self.open_index_in_viewer(ctx, index);
            return;
        }

        if should_loop_at_end(self.slideshow.end_behavior) {
            let first_index = self
                .service
                .navigate_gallery(NavigationDirection::First)
                .ok()
                .flatten();
            if let Some(index) = first_index {
                self.open_index_in_viewer(ctx, index);
            }
            return;
        }

        self.pause_slideshow();
    }

    fn open_index_in_viewer(&mut self, ctx: &Context, index: usize) {
        let Ok(Some((path, fit_to_window))) =
            self.service.get_gallery_image_path_and_fit_if_viewer(index)
        else {
            return;
        };
        self.open_image(ctx, &path, fit_to_window);
    }
}

fn should_loop_at_end(end_behavior: SlideshowEndBehavior) -> bool {
    matches!(end_behavior, SlideshowEndBehavior::Loop)
}

fn normalized_interval(interval_seconds: u64) -> u64 {
    interval_seconds.max(1)
}

#[cfg(test)]
mod tests {
    use super::{normalized_interval, should_loop_at_end};
    use crate::adapters::egui::app::types::SlideshowEndBehavior;

    #[test]
    fn slideshow_end_behavior_looping() {
        assert!(should_loop_at_end(SlideshowEndBehavior::Loop));
        assert!(!should_loop_at_end(SlideshowEndBehavior::Stop));
    }

    #[test]
    fn slideshow_interval_is_normalized() {
        assert_eq!(normalized_interval(0), 1);
        assert_eq!(normalized_interval(1), 1);
        assert_eq!(normalized_interval(5), 5);
    }
}
