    matches!(byte, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-')
}


fn find_next_mention_token_range(text: &str, token: &str, from: usize) -> Option<Range<usize>> {
    if token.is_empty() || from >= text.len() {
        return None;
    }
    let bytes = text.as_bytes();
    let token_bytes = token.as_bytes();
    let mut index = from;


    while index < bytes.len() {
        if bytes[index] != b'$' {
            index += 1;
            continue;
        }


        let end = index.saturating_add(token_bytes.len());
        if end > bytes.len() {
            return None;
        }
        if &bytes[index..end] != token_bytes {
            index += 1;
            continue;
        }


        if bytes
            .get(end)
            .is_none_or(|byte| !is_mention_name_char(*byte))
        {
            return Some(index..end);
        }


        index = end;
    }


    None
}


impl Renderable for ChatComposer {
    fn cursor_pos(&self, area: Rect) -> Option<(u16, u16)> {
        if !self.input_enabled || self.selected_remote_image_index.is_some() {
            return None;
        }


        if let Some(pos) = self.history_search_cursor_pos(area) {
            return Some(pos);
        }


        let [_, _, textarea_rect, _] = self.layout_areas(area);
        let state = *self.textarea_state.borrow();
        self.textarea.cursor_pos_with_state(textarea_rect, state)
    }


    fn cursor_style(&self, _area: Rect) -> crossterm::cursor::SetCursorStyle {
        if self.textarea.uses_vim_insert_cursor() {
            crossterm::cursor::SetCursorStyle::SteadyBar
        } else {
            crossterm::cursor::SetCursorStyle::SteadyBlock
        }
    }


    fn desired_height(&self, width: u16) -> u16 {
        let footer_props = self.footer_props();
        let footer_hint_height = self
            .custom_footer_height()
            .unwrap_or_else(|| footer_height(&footer_props));
        let footer_spacing = Self::footer_spacing(footer_hint_height);
        let footer_total_height = footer_hint_height + footer_spacing;
        const COLS_WITH_MARGIN: u16 = LIVE_PREFIX_COLS + 1;
        let inner_width = width.saturating_sub(COLS_WITH_MARGIN);
        let remote_images_height: u16 = self
            .remote_images_lines(inner_width)
            .len()
            .try_into()
            .unwrap_or(u16::MAX);
        let remote_images_separator = u16::from(remote_images_height > 0);
        self.textarea.desired_height(inner_width)
            + remote_images_height
            + remote_images_separator
            + 2
            + match &self.active_popup {
