use egui::{Color32, Context, Theme, ThemePreference, Visuals};

use super::state::ThemeChoice;

pub const MIN_TEXT_SCALE: f32 = 0.75;
pub const MAX_TEXT_SCALE: f32 = 2.0;

// egui's default grey body text falls short of WCAG AA on some panels; status text must stay
// readable in both themes, so body text is pushed close to black or white.
pub fn visuals(theme: Theme) -> Visuals {
    let mut visuals = theme.default_visuals();
    // The material box keeps focus; a blinking caret would wake an idle window twice a second.
    visuals.text_cursor.blink = false;
    visuals.override_text_color = Some(match theme {
        Theme::Dark => Color32::from_gray(235),
        Theme::Light => Color32::from_gray(15),
    });
    visuals
}

pub fn install(context: &Context) {
    for theme in [Theme::Dark, Theme::Light] {
        context.set_visuals_of(theme, visuals(theme));
    }
    context.all_styles_mut(|style| style.interaction.selectable_labels = true);
}

pub fn apply(context: &Context, theme: ThemeChoice, text_scale: f32) {
    context.set_theme(match theme {
        ThemeChoice::System => ThemePreference::System,
        ThemeChoice::Light => ThemePreference::Light,
        ThemeChoice::Dark => ThemePreference::Dark,
    });
    let scale = text_scale.clamp(MIN_TEXT_SCALE, MAX_TEXT_SCALE);
    if (context.zoom_factor() - scale).abs() > f32::EPSILON {
        context.set_zoom_factor(scale);
    }
}

pub fn contrast_ratio(foreground: Color32, background: Color32) -> f32 {
    let (lighter, darker) = {
        let (a, b) = (luminance(foreground), luminance(background));
        if a >= b { (a, b) } else { (b, a) }
    };
    (lighter + 0.05) / (darker + 0.05)
}

fn luminance(color: Color32) -> f32 {
    let channel = |value: u8| {
        let value = f32::from(value) / 255.0;
        if value <= 0.039_28 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_text_meets_wcag_aaa_contrast_in_both_themes() {
        for theme in [Theme::Light, Theme::Dark] {
            let visuals = visuals(theme);
            let text = visuals.text_color();
            for background in [
                visuals.panel_fill,
                visuals.window_fill,
                visuals.extreme_bg_color,
            ] {
                let ratio = contrast_ratio(text, background);
                assert!(ratio >= 7.0, "{theme:?}: {ratio:.2} on {background:?}");
            }
        }
    }

    #[test]
    fn contrast_ratio_matches_the_wcag_reference_points() {
        assert!((contrast_ratio(Color32::BLACK, Color32::WHITE) - 21.0).abs() < 0.01);
        assert!((contrast_ratio(Color32::WHITE, Color32::WHITE) - 1.0).abs() < 0.01);
    }

    #[test]
    fn theme_and_text_scale_apply_to_the_context_within_bounds() {
        let context = Context::default();
        install(&context);
        let pass = |context: &Context| {
            context
                .run_ui(egui::RawInput::default(), |_| {})
                .textures_delta
                .clear();
        };
        apply(&context, ThemeChoice::Dark, 5.0);
        pass(&context);
        assert_eq!(
            context.options(|options| options.theme_preference),
            ThemePreference::Dark
        );
        assert!((context.zoom_factor() - MAX_TEXT_SCALE).abs() < f32::EPSILON);
        apply(&context, ThemeChoice::Light, 1.25);
        pass(&context);
        assert_eq!(context.theme(), Theme::Light);
        assert!((context.zoom_factor() - 1.25).abs() < f32::EPSILON);
        assert!(context.global_style().interaction.selectable_labels);
    }
}
