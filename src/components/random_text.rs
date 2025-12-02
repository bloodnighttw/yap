use std::hash::{BuildHasher, Hash, Hasher};
use std::collections::hash_map::RandomState;

use crate::framework::Updater;

#[derive(Default)]
pub struct RandomText {
    task_handle: Option<tokio::task::JoinHandle<()>>,
}

struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new() -> Self {
        let random_state = RandomState::new();
        let mut hasher = random_state.build_hasher();
        std::time::SystemTime::now().hash(&mut hasher);

        SimpleRng {
            state: hasher.finish(),
        }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.state
    }

    fn next_range(&mut self, max: usize) -> usize {
        (self.next() % max as u64) as usize
    }
}

fn generate_random_text(length: usize) -> String {
    let mut rng = SimpleRng::new();
    let chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 ";
    let chars_vec: Vec<char> = chars.chars().collect();

    (0..length)
        .map(|_| chars_vec[rng.next_range(chars_vec.len())])
        .collect()
}

impl crate::framework::Component for RandomText {
    fn component_did_mount(
        &mut self,
        _area: ratatui::layout::Size,
        updater: Updater,
    ) -> color_eyre::Result<()> {
        self.task_handle = Some(tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
                updater.update();
            }
        }));
        Ok(())
    }

    fn render(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
    ) -> color_eyre::Result<()> {
        let rand = generate_random_text(50);

        let paragraph = ratatui::widgets::Paragraph::new(ratatui::text::Span::from(rand));
        frame.render_widget(paragraph, area);
        Ok(())
    }
}

impl Drop for RandomText {
    fn drop(&mut self) {
        if let Some(handle) = self.task_handle.take() {
            handle.abort();
        }
    }
}
