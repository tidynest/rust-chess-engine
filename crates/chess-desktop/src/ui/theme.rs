use egui::Color32;

/// Design token system for the chess application
/// All visual styling goes through this theme to ensure consistency
#[derive(Clone, Debug)]
pub struct Theme {
    // === COLOR TOKENS ===

    // Primary UI Colors
    pub background: Color32,
    pub surface: Color32,
    pub primary: Color32,
    pub accent: Color32,

    // Text Colors
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_disabled: Color32,

    // Board Colors
    pub board_light: Color32,
    pub board_dark: Color32,

    // Evaluation Bar
    pub eval_white: Color32,
    pub eval_black: Color32,
    pub eval_text: Color32,

    // Interactive States
    pub hover: Color32,
    pub selected: Color32,
    pub legal_move: Color32,
    pub check: Color32,

    // Status Colors
    pub success: Color32,
    pub warning: Color32,
    pub error: Color32,

    // === SPACING TOKENS (8pt Grid) ===
    pub space_xs: f32,  // 8px  - Tight spacing
    pub space_sm: f32,  // 16px - Standard spacing
    pub space_md: f32,  // 24px - Comfortable spacing
    pub space_lg: f32,  // 32px - Section spacing
    pub space_xl: f32,  // 40px - Large separation
    pub space_2xl: f32, // 48px - Major sections

    // === TYPOGRAPHY TOKENS ===
    pub font_size_xs: f32,  // 12pt - Labels, captions
    pub font_size_sm: f32,  // 14pt - Secondary text
    pub font_size_md: f32,  // 16pt - Body text, buttons
    pub font_size_lg: f32,  // 20pt - Section headers
    pub font_size_xl: f32,  // 24pt - Player names
    pub font_size_2xl: f32, // 32pt - Title

    // === COMPONENT TOKENS ===
    pub button_height: f32,
    pub panel_padding: f32,
    pub border_radius: f32,
}

impl Theme {
    /// Classic Monochrome Premium Theme
    /// Timeless, sophisticated, excellent contrast
    /// Perfect for focused, distraction-free play
    pub fn classic_monochrome() -> Self {
        Self {
            // Primary UI Colors
            background: Color32::from_rgb(250, 250, 250), // Off-white #FAFAFA
            surface: Color32::from_rgb(255, 255, 255),    // White
            primary: Color32::from_rgb(44, 95, 61),       // Deep green #2C5F3D
            accent: Color32::from_rgb(44, 95, 61),        // Deep green (same as primary)

            // Text Colors
            text_primary: Color32::from_rgb(26, 26, 26), // Near-black #1A1A1A
            text_secondary: Color32::from_rgb(100, 100, 100), // Gray #646464
            text_disabled: Color32::from_rgb(180, 180, 180), // Light gray

            // Board Colors
            board_light: Color32::from_rgb(245, 245, 220), // Cream #F5F5DC
            board_dark: Color32::from_rgb(139, 123, 107),  // Warm gray #8B7B6B

            // Evaluation Bar
            eval_white: Color32::from_rgb(245, 245, 220), // Match light square
            eval_black: Color32::from_rgb(60, 60, 60),    // Dark gray
            eval_text: Color32::from_rgb(26, 26, 26),     // Near-black

            // Interactive States
            hover: Color32::from_rgba_premultiplied(44, 95, 61, 40), // 15% primary
            selected: Color32::from_rgba_premultiplied(44, 95, 61, 60), // 25% primary
            legal_move: Color32::from_rgba_premultiplied(44, 95, 61, 80), // 30% primary
            check: Color32::from_rgb(200, 50, 50),                   // Red for check

            // Status Colors
            success: Color32::from_rgb(76, 175, 80), // Green
            warning: Color32::from_rgb(255, 152, 0), // Orange
            error: Color32::from_rgb(244, 67, 54),   // Red

            // Spacing (8pt grid)
            space_xs: 8.0,
            space_sm: 16.0,
            space_md: 24.0,
            space_lg: 32.0,
            space_xl: 40.0,
            space_2xl: 48.0,

            // Typography
            font_size_xs: 12.0,
            font_size_sm: 14.0,
            font_size_md: 16.0,
            font_size_lg: 20.0,
            font_size_xl: 24.0,
            font_size_2xl: 32.0,

            // Components
            button_height: 32.0,
            panel_padding: 16.0,
            border_radius: 4.0,
        }
    }

    /// Warm Minimal Theme
    /// Organic, welcoming feel that reduces eye strain
    /// Perfect for extended play sessions
    pub fn warm_minimal() -> Self {
        Self {
            // Primary UI Colors
            background: Color32::from_rgb(245, 240, 232), // Soft beige #F5F0E8
            surface: Color32::from_rgb(255, 250, 245),    // Off-white
            primary: Color32::from_rgb(201, 147, 131),    // Terracotta #C99383
            accent: Color32::from_rgb(159, 181, 159),     // Sage green #9FB59F

            // Text Colors
            text_primary: Color32::from_rgb(61, 40, 23), // Dark brown #3D2817
            text_secondary: Color32::from_rgb(120, 100, 80), // Medium brown
            text_disabled: Color32::from_rgb(180, 170, 160), // Light brown

            // Board Colors
            board_light: Color32::from_rgb(255, 255, 240), // Ivory #FFFFF0
            board_dark: Color32::from_rgb(107, 78, 61),    // Walnut #6B4E3D

            // Evaluation Bar
            eval_white: Color32::from_rgb(255, 255, 240), // Match light square
            eval_black: Color32::from_rgb(80, 60, 50),    // Darker walnut
            eval_text: Color32::from_rgb(61, 40, 23),     // Dark brown

            // Interactive States
            hover: Color32::from_rgba_premultiplied(201, 147, 131, 40),
            selected: Color32::from_rgba_premultiplied(201, 147, 131, 60),
            legal_move: Color32::from_rgba_premultiplied(159, 181, 159, 100),
            check: Color32::from_rgb(180, 70, 70), // Muted red

            // Status Colors
            success: Color32::from_rgb(120, 160, 100), // Muted green
            warning: Color32::from_rgb(210, 140, 80),  // Muted orange
            error: Color32::from_rgb(200, 90, 80),     // Muted red

            // Spacing (8pt grid)
            space_xs: 8.0,
            space_sm: 16.0,
            space_md: 24.0,
            space_lg: 32.0,
            space_xl: 40.0,
            space_2xl: 48.0,

            // Typography
            font_size_xs: 12.0,
            font_size_sm: 14.0,
            font_size_md: 16.0,
            font_size_lg: 20.0,
            font_size_xl: 24.0,
            font_size_2xl: 32.0,

            // Components
            button_height: 32.0,
            panel_padding: 16.0,
            border_radius: 4.0,
        }
    }

    /// Modern Dark Theme
    /// Sleek, professional look perfect for night play
    /// High contrast with sophisticated color palette
    pub fn modern_dark() -> Self {
        Self {
            // Primary UI Colors
            background: Color32::from_rgb(30, 30, 30), // Rich black #1E1E1E
            surface: Color32::from_rgb(40, 40, 45),    // Slightly lighter
            primary: Color32::from_rgb(212, 175, 55),  // Gold #D4AF37
            accent: Color32::from_rgb(74, 155, 155),   // Teal #4A9B9B

            // Text Colors
            text_primary: Color32::from_rgb(232, 232, 232), // Off-white #E8E8E8
            text_secondary: Color32::from_rgb(160, 160, 160), // Medium gray
            text_disabled: Color32::from_rgb(100, 100, 100), // Dark gray

            // Board Colors
            board_light: Color32::from_rgb(212, 212, 212), // Light gray #D4D4D4
            board_dark: Color32::from_rgb(74, 74, 74),     // Dark gray #4A4A4A

            // Evaluation Bar
            eval_white: Color32::from_rgb(220, 220, 220), // Light gray
            eval_black: Color32::from_rgb(50, 50, 50),    // Very dark gray
            eval_text: Color32::from_rgb(232, 232, 232),  // Off-white

            // Interactive States
            hover: Color32::from_rgba_premultiplied(212, 175, 55, 40),
            selected: Color32::from_rgba_premultiplied(212, 175, 55, 60),
            legal_move: Color32::from_rgba_premultiplied(74, 155, 155, 80),
            check: Color32::from_rgb(220, 80, 80), // Bright red

            // Status Colors
            success: Color32::from_rgb(100, 200, 100), // Bright green
            warning: Color32::from_rgb(255, 180, 50),  // Bright orange
            error: Color32::from_rgb(255, 90, 90),     // Bright red

            // Spacing (8pt grid)
            space_xs: 8.0,
            space_sm: 16.0,
            space_md: 24.0,
            space_lg: 32.0,
            space_xl: 40.0,
            space_2xl: 48.0,

            // Typography
            font_size_xs: 12.0,
            font_size_sm: 14.0,
            font_size_md: 16.0,
            font_size_lg: 20.0,
            font_size_xl: 24.0,
            font_size_2xl: 32.0,

            // Components
            button_height: 32.0,
            panel_padding: 16.0,
            border_radius: 4.0,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::classic_monochrome()
    }
}

/// Available theme variants
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeVariant {
    ClassicMonochrome,
    WarmMinimal,
    ModernDark,
}

impl ThemeVariant {
    pub fn name(&self) -> &'static str {
        match self {
            Self::ClassicMonochrome => "Classic Monochrome",
            Self::WarmMinimal => "Warm Minimal",
            Self::ModernDark => "Modern Dark",
        }
    }

    pub fn to_theme(&self) -> Theme {
        match self {
            Self::ClassicMonochrome => Theme::classic_monochrome(),
            Self::WarmMinimal => Theme::warm_minimal(),
            Self::ModernDark => Theme::modern_dark(),
        }
    }

    pub fn all() -> [ThemeVariant; 3] {
        [Self::ClassicMonochrome, Self::WarmMinimal, Self::ModernDark]
    }
}
