use teloxide::payloads::{EditMessageTextSetters, SendMessageSetters};
use teloxide::requests::Requester;
use teloxide::types::ParseMode;

/// Convert AI-generated Markdown to Telegram HTML.
///
/// Telegram HTML supports: <b>, <i>, <u>, <s>, <code>, <pre>, <a>, <blockquote>
/// AI models typically output standard Markdown which we convert here.
pub fn markdown_to_telegram_html(text: &str) -> String {
    let mut result = String::with_capacity(text.len() * 2);
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;
    let mut in_code_block = false;
    let mut code_block_lang = String::new();
    let mut code_block_content = String::new();

    while i < lines.len() {
        let line = lines[i];

        // Handle fenced code blocks
        if line.trim_start().starts_with("```") {
            if in_code_block {
                // Close code block
                let escaped = escape_html(&code_block_content);
                if code_block_lang.is_empty() {
                    result.push_str(&format!("<pre>{}</pre>", escaped.trim_end()));
                } else {
                    result.push_str(&format!(
                        "<pre><code class=\"language-{}\">{}</code></pre>",
                        escape_html(&code_block_lang),
                        escaped.trim_end()
                    ));
                }
                result.push('\n');
                in_code_block = false;
                code_block_lang.clear();
                code_block_content.clear();
            } else {
                // Open code block
                in_code_block = true;
                code_block_lang = line.trim_start().trim_start_matches('`').trim().to_string();
            }
            i += 1;
            continue;
        }

        if in_code_block {
            if !code_block_content.is_empty() {
                code_block_content.push('\n');
            }
            code_block_content.push_str(line);
            i += 1;
            continue;
        }

        // Headings → bold text
        if let Some(heading) = parse_heading(line) {
            if !result.is_empty() && !result.ends_with('\n') {
                result.push('\n');
            }
            result.push_str(&format!("<b>{}</b>\n", convert_inline(&heading)));
            i += 1;
            continue;
        }

        // Horizontal rules
        let trimmed = line.trim();
        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            result.push_str("───────────────\n");
            i += 1;
            continue;
        }

        // Blockquotes
        if trimmed.starts_with("> ") {
            let quote_content = trimmed.strip_prefix("> ").unwrap_or(trimmed);
            result.push_str(&format!(
                "<blockquote>{}</blockquote>\n",
                convert_inline(quote_content)
            ));
            i += 1;
            continue;
        }

        // Unordered list items: - or * or •
        if let Some(item) = parse_list_item(trimmed) {
            result.push_str(&format!("  • {}\n", convert_inline(&item)));
            i += 1;
            continue;
        }

        // Ordered list items: 1. 2. etc.
        if let Some((num, item)) = parse_ordered_list_item(trimmed) {
            result.push_str(&format!("  {}. {}\n", num, convert_inline(&item)));
            i += 1;
            continue;
        }

        // Regular paragraph line
        if trimmed.is_empty() {
            result.push('\n');
        } else {
            result.push_str(&convert_inline(line));
            result.push('\n');
        }

        i += 1;
    }

    // Close unclosed code block
    if in_code_block {
        let escaped = escape_html(&code_block_content);
        result.push_str(&format!("<pre>{}</pre>\n", escaped.trim_end()));
    }

    result.trim_end().to_string()
}

/// Parse a Markdown heading line, return the heading text without the # prefix.
fn parse_heading(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("### ") {
        Some(trimmed[4..].to_string())
    } else if trimmed.starts_with("## ") {
        Some(trimmed[3..].to_string())
    } else if trimmed.starts_with("# ") {
        Some(trimmed[2..].to_string())
    } else {
        None
    }
}

/// Parse an unordered list item, return the item text.
fn parse_list_item(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("- ") {
        Some(trimmed[2..].to_string())
    } else if trimmed.starts_with("* ") && !trimmed.starts_with("**") {
        Some(trimmed[2..].to_string())
    } else if trimmed.starts_with("• ") {
        Some(trimmed.chars().skip(2).collect())
    } else {
        None
    }
}

/// Parse an ordered list item like "1. text", return (number, text).
fn parse_ordered_list_item(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim_start();
    let dot_pos = trimmed.find(". ")?;
    let num_part = &trimmed[..dot_pos];
    if num_part.chars().all(|c| c.is_ascii_digit()) && !num_part.is_empty() {
        Some((num_part.to_string(), trimmed[dot_pos + 2..].to_string()))
    } else {
        None
    }
}

/// Convert inline Markdown formatting to Telegram HTML.
/// Handles: **bold**, *italic*, `code`, [links](url), ~~strikethrough~~
fn convert_inline(text: &str) -> String {
    let mut result = String::with_capacity(text.len() * 2);
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Inline code: `text`
        if chars[i] == '`' && !matches!(chars.get(i + 1), Some('`')) {
            if let Some(end) = find_closing(&chars, i + 1, '`') {
                let code_text: String = chars[i + 1..end].iter().collect();
                result.push_str(&format!("<code>{}</code>", escape_html(&code_text)));
                i = end + 1;
                continue;
            }
        }

        // Bold: **text**
        if i + 1 < len && chars[i] == '*' && chars[i + 1] == '*' {
            if let Some(end) = find_double_closing(&chars, i + 2, '*') {
                let inner: String = chars[i + 2..end].iter().collect();
                result.push_str(&format!("<b>{}</b>", convert_inline(&inner)));
                i = end + 2;
                continue;
            }
        }

        // Italic: *text* (single asterisk, not followed by another)
        if chars[i] == '*' && (i + 1 >= len || chars[i + 1] != '*') {
            if let Some(end) = find_closing_single_star(&chars, i + 1) {
                let inner: String = chars[i + 1..end].iter().collect();
                result.push_str(&format!("<i>{}</i>", convert_inline(&inner)));
                i = end + 1;
                continue;
            }
        }

        // Strikethrough: ~~text~~
        if i + 1 < len && chars[i] == '~' && chars[i + 1] == '~' {
            if let Some(end) = find_double_closing(&chars, i + 2, '~') {
                let inner: String = chars[i + 2..end].iter().collect();
                result.push_str(&format!("<s>{}</s>", convert_inline(&inner)));
                i = end + 2;
                continue;
            }
        }

        // Links: [text](url)
        if chars[i] == '[' {
            if let Some((link_text, url, end_pos)) = parse_link(&chars, i) {
                result.push_str(&format!(
                    "<a href=\"{}\">{}</a>",
                    escape_html(&url),
                    escape_html(&link_text)
                ));
                i = end_pos;
                continue;
            }
        }

        // Regular character — escape HTML
        let ch = chars[i];
        match ch {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            _ => result.push(ch),
        }
        i += 1;
    }

    result
}

/// Escape HTML special characters.
pub fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn find_closing(chars: &[char], start: usize, marker: char) -> Option<usize> {
    for i in start..chars.len() {
        if chars[i] == marker {
            return Some(i);
        }
    }
    None
}

fn find_closing_single_star(chars: &[char], start: usize) -> Option<usize> {
    for i in start..chars.len() {
        if chars[i] == '*' && (i + 1 >= chars.len() || chars[i + 1] != '*') {
            // Ensure there's actual content between the stars
            if i > start {
                return Some(i);
            }
        }
    }
    None
}

fn find_double_closing(chars: &[char], start: usize, marker: char) -> Option<usize> {
    let len = chars.len();
    for i in start..len.saturating_sub(1) {
        if chars[i] == marker && chars[i + 1] == marker {
            if i > start {
                return Some(i);
            }
        }
    }
    None
}

fn parse_link(chars: &[char], start: usize) -> Option<(String, String, usize)> {
    // [text](url)
    let len = chars.len();
    if start >= len || chars[start] != '[' {
        return None;
    }
    let mut i = start + 1;
    let mut bracket_depth = 1;
    // Find closing ]
    while i < len && bracket_depth > 0 {
        if chars[i] == '[' {
            bracket_depth += 1;
        } else if chars[i] == ']' {
            bracket_depth -= 1;
        }
        if bracket_depth > 0 {
            i += 1;
        }
    }
    if i >= len || chars[i] != ']' {
        return None;
    }
    let text: String = chars[start + 1..i].iter().collect();
    i += 1;
    if i >= len || chars[i] != '(' {
        return None;
    }
    i += 1;
    let url_start = i;
    let mut paren_depth = 1;
    while i < len && paren_depth > 0 {
        if chars[i] == '(' {
            paren_depth += 1;
        } else if chars[i] == ')' {
            paren_depth -= 1;
        }
        if paren_depth > 0 {
            i += 1;
        }
    }
    if i >= len {
        return None;
    }
    let url: String = chars[url_start..i].iter().collect();
    Some((text, url, i + 1))
}

/// Split a message into chunks that fit Telegram's 4096 char limit.
/// Splits at paragraph boundaries when possible.
pub fn split_message(text: &str, max_len: usize) -> Vec<String> {
    if text.len() <= max_len {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut remaining = text;

    while remaining.len() > max_len {
        let split_at = remaining[..max_len]
            .rfind("\n\n")
            .or_else(|| remaining[..max_len].rfind('\n'))
            .unwrap_or(max_len);

        chunks.push(remaining[..split_at].to_string());
        remaining = remaining[split_at..].trim_start();
    }

    if !remaining.is_empty() {
        chunks.push(remaining.to_string());
    }

    chunks
}

/// Format a model response with a bold header and convert content to HTML.
pub fn format_model_response(display_name: &str, content: &str) -> String {
    let html_content = markdown_to_telegram_html(content);
    format!("<b>💬 {}</b>\n\n{}", escape_html(display_name), html_content)
}

/// Format the master verdict with a distinctive header.
pub fn format_master_verdict(content: &str) -> String {
    let html_content = markdown_to_telegram_html(content);
    format!("<b>⚖️ MASTER VERDICT</b>\n\n{}", html_content)
}

/// Format a "thinking" status message.
pub fn format_thinking(display_name: &str) -> String {
    format!("🧠 <b>{}</b> is thinking...", escape_html(display_name))
}

/// Send a formatted HTML message, splitting if needed.
/// Falls back to plain text if HTML parsing fails.
pub async fn send_html(
    bot: &teloxide::Bot,
    chat_id: teloxide::types::ChatId,
    html: &str,
) -> Result<teloxide::types::Message, teloxide::RequestError> {
    let chunks = split_message(html, 4096);
    let mut last_msg = None;

    for chunk in &chunks {
        let result = bot
            .send_message(chat_id, chunk)
            .parse_mode(ParseMode::Html)
            .await;

        match result {
            Ok(msg) => last_msg = Some(msg),
            Err(_) => {
                // Fallback to plain text if HTML fails
                last_msg = Some(bot.send_message(chat_id, strip_html(chunk)).await?);
            }
        }
    }

    Ok(last_msg.unwrap_or_else(|| unreachable!("at least one chunk")))
}

/// Edit a message with HTML formatting, falling back to plain text.
pub async fn edit_html(
    bot: &teloxide::Bot,
    chat_id: teloxide::types::ChatId,
    message_id: teloxide::types::MessageId,
    html: &str,
) -> Result<(), teloxide::RequestError> {
    let chunks = split_message(html, 4096);

    // Edit the original message with the first chunk
    let result: Result<teloxide::types::Message, _> = bot
        .edit_message_text(chat_id, message_id, &chunks[0])
        .parse_mode(ParseMode::Html)
        .await;

    if result.is_err() {
        // Fallback to plain text — if this also fails, log the error
        let fallback: Result<teloxide::types::Message, _> = bot
            .edit_message_text(chat_id, message_id, &strip_html(&chunks[0]))
            .await;
        if let Err(e) = fallback {
            log::error!("Failed to edit message (both HTML and plain text): {}", e);
        }
    }

    // Send remaining chunks as new messages
    for chunk in &chunks[1..] {
        let _ = send_html(bot, chat_id, chunk).await;
    }

    Ok(())
}

/// Send typing indicator + "thinking" message with HTML formatting.
pub async fn send_thinking(
    bot: &teloxide::Bot,
    chat_id: teloxide::types::ChatId,
    display_name: &str,
) -> Result<teloxide::types::Message, teloxide::RequestError> {
    // Show "typing..." bubble animation
    let _ = bot
        .send_chat_action(chat_id, teloxide::types::ChatAction::Typing)
        .await;

    bot.send_message(chat_id, format_thinking(display_name))
        .parse_mode(ParseMode::Html)
        .await
}

/// Send just the typing indicator (no message).
/// Call this periodically during long operations to keep the indicator alive.
pub async fn send_typing(
    bot: &teloxide::Bot,
    chat_id: teloxide::types::ChatId,
) {
    let _ = bot
        .send_chat_action(chat_id, teloxide::types::ChatAction::Typing)
        .await;
}

/// Strip HTML tags for fallback plain text.
fn strip_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(ch);
        }
    }
    result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

/// Run an async future while continuously sending the typing indicator.
/// The typing indicator auto-expires after ~5s, so we resend every 4s.
pub async fn with_typing<F, T>(
    bot: &teloxide::Bot,
    chat_id: teloxide::types::ChatId,
    future: F,
) -> T
where
    F: std::future::Future<Output = T>,
{
    let bot_clone = bot.clone();
    let typing_handle = tokio::spawn(async move {
        loop {
            let _ = bot_clone
                .send_chat_action(chat_id, teloxide::types::ChatAction::Typing)
                .await;
            tokio::time::sleep(std::time::Duration::from_secs(4)).await;
        }
    });

    let result = future.await;
    typing_handle.abort();
    result
}
