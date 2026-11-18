//! Non-interactive visual proof for the home layout. No model calls or transcript mutation.

use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

const BG_CARD: Color = Color::Rgb(36, 40, 59);
const BG_ELEVATED: Color = Color::Rgb(41, 46, 66);
const BORDER: Color = Color::Rgb(65, 72, 104);
const TEXT: Color = Color::Rgb(192, 202, 245);
const MUTED: Color = Color::Rgb(169, 177, 214);
const ACCENT: Color = Color::Rgb(187, 154, 247);

/// The composer lives above center on a fresh session. Position becomes bottom-docked
/// in the future CHAT state; this preview intentionally does NOT claim that transition.
pub fn home_composer_area(area: Rect) -> Rect {
    let width = area.width.saturating_sub(4).min(84);
    let height = 6_u16.min(area.height.saturating_sub(3));
    let x = area.x + area.width.saturating_sub(width) / 2;
    let desired_y = area.y + area.height.saturating_mul(48) / 100;
    let max_y = area.y + area.height.saturating_sub(height).saturating_sub(2);
    Rect {
        x,
        y: desired_y.min(max_y),
        width,
        height,
    }
}

pub fn draw_home(frame: &mut Frame<'_>, directory: &str) {
    let area = frame.area();
    if area.width < 24 || area.height < 12 {
        frame.render_widget(
            Paragraph::new("PI VISUAL · enlarge terminal · q to quit")
                .style(Style::default().fg(MUTED)),
            area,
        );
        return;
    }

    let heading = Rect {
        x: area.x,
        y: area.y + area.height / 5,
        width: area.width,
        height: 6,
    };
    let logo = "  ____    ___\n |  _ \\  |_ _|\n | |_) |  | |\n |  __/   | |\n |_|     |___|";
    frame.render_widget(
        Paragraph::new(logo)
            .alignment(Alignment::Center)
            .style(Style::default().fg(ACCENT)),
        heading,
    );

    let card = home_composer_area(area);
    if card.width > 0 && card.height > 0 {
        let border = Block::default()
            .title(" Pi · local preview ")
            .title_style(Style::default().fg(ACCENT))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(BORDER))
            .style(Style::default().bg(BG_CARD));
        frame.render_widget(border, card);

        if card.width > 4 && card.height > 3 {
            let body = Rect {
                x: card.x + 2,
                y: card.y + 1,
                width: card.width - 4,
                height: card.height - 2,
            };
            frame.render_widget(
                Paragraph::new(vec![
                    Line::from(Span::styled(
                        "Ask anything…  \"Fix a TODO in the codebase\"",
                        Style::default().fg(MUTED),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Preview only  ·  No model connected",
                        Style::default().fg(ACCENT).bg(BG_ELEVATED),
                    )),
                ])
                .style(Style::default().fg(TEXT).bg(BG_CARD)),
                body,
            );
        }
    }
    let footer = Rect {
        x: area.x + 1,
        y: area.bottom().saturating_sub(1),
        width: area.width.saturating_sub(2),
        height: 1,
    };
    let right = "Cost —  ·  Context —"; // Unknown: no fabricated model stats.
    let right_width = right.chars().count() as u16;
    let gap = 2u16;
    let available = footer.width;
    if available >= right_width + gap {
        let left_width = available - right_width - gap;
        let left: String = directory.chars().take(usize::from(left_width)).collect();
        let left_area = Rect {
            x: footer.x,
            y: footer.y,
            width: left_width,
            height: 1,
        };
        let right_area = Rect {
            x: footer.x + left_width + gap,
            y: footer.y,
            width: right_width,
            height: 1,
        };
        frame.render_widget(
            Paragraph::new(left).style(Style::default().fg(MUTED)),
            left_area,
        );
        frame.render_widget(
            Paragraph::new(right)
                .alignment(Alignment::Right)
                .style(Style::default().fg(MUTED)),
            right_area,
        );
    } else {
        frame.render_widget(
            Paragraph::new("Context —")
                .alignment(Alignment::Right)
                .style(Style::default().fg(MUTED)),
            footer,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_composer_stays_inside_normal_viewport() {
        for width in [40, 60, 80, 120, 160] {
            for height in [16, 18, 24, 40] {
                let full = Rect::new(0, 0, width, height);
                let r = home_composer_area(full);
                assert!(r.x + r.width <= full.right());
                assert!(r.y + r.height <= full.bottom());
            }
        }
    }

    #[test]
    fn narrow_composer_does_not_underflow() {
        let r = home_composer_area(Rect::new(0, 0, 1, 1));
        assert_eq!(r.width, 0);
        assert_eq!(r.height, 0);
    }
}
