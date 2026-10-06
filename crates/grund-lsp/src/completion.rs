// Completion transport and UTF-16 marshaling (§FS-lsp.1.6.2, §FS-lsp.1.6.3).

impl Server {
    /// Initial project/member characters, deduplicated across folders
    /// (§FS-lsp.1.6.1). New configuration characters require client restart.
    fn completion_trigger_characters(&self) -> Vec<String> {
        self.projects.iter().filter_map(|project| project.completion.as_ref())
            .flat_map(grund_core::LspCompletionContext::trigger_characters)
            .collect::<BTreeSet<_>>().into_iter().collect()
    }

    /// Marshal core byte edits without introducing protocol-side recognition
    /// or rescanning (§AR-lsp.1, §FS-lsp.1.6.3, §FS-lsp.1.6.4).
    fn completion(&self, params: Value) -> Result<lsp_types::CompletionList> {
        let params: lsp_types::CompletionParams = serde_json::from_value(params)?;
        let position = params.text_document_position.position;
        let uri = params.text_document_position.text_document.uri;
        let mut result = lsp_types::CompletionList { is_incomplete: true, items: Vec::new() };
        let (Some(path), Some(text)) = (uri.to_file_path().ok(), self.document_text(&uri)) else {
            return Ok(result);
        };
        let Some(context) = self.project_for_path(&path)
            .and_then(|project| project.completion.as_ref()) else { return Ok(result) };
        let Some(line) = text.lines().nth(position.line as usize) else { return Ok(result) };
        let cursor = utf16_to_byte(line, position.character);
        let at = |byte| Position { line: position.line, character: byte_to_utf16(line, byte) };
        result.items = context.complete(&path, &text, position.line as usize, cursor)
            .into_iter().map(|candidate| lsp_types::CompletionItem {
                label: candidate.id,
                detail: Some(format!("{} — {}", candidate.title, candidate.source_path)),
                sort_text: Some(candidate.sort_text),
                filter_text: Some(candidate.filter_text),
                insert_text_format: Some(lsp_types::InsertTextFormat::PLAIN_TEXT),
                text_edit: Some(lsp_types::CompletionTextEdit::Edit(TextEdit {
                    range: Range { start: at(candidate.start), end: at(candidate.end) },
                    new_text: candidate.text,
                })),
                ..Default::default()
            }).collect();
        Ok(result)
    }
}
