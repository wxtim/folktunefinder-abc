use svg;
use tune_ast_three;
use abc_lexer as l;
use music;

/// Desired stave width.
/// TODO update this when there are real glyphs.
const STAVE_WIDTH: f32 = 800.0;

// Height of a single note head;
const HEAD_HEIGHT: f32 = 10.0;
const HEAD_WIDTH: f32 = HEAD_HEIGHT * 1.25;

const STEM_HEIGHT: f32 = 40.0;

// Vertical padding between each stave.
const STAVE_V_MARGIN: f32 = 20.0;

// Vertical padding between each System.
const SYSTEM_V_MARGIN: f32 = 20.0;

// How many lines (including spaces) in a stave.
const LINES_IN_STAVE: i32 = 9;

// If the scale is below this (i.e. we won't fill the line) then use the natural stave length.
// Prevents non-full-width staves from being forced to be full width.
const MINIMUM_STAVE_SCALE: f32 = 1.8;

pub struct Typesetting {}

impl Typesetting {
    pub fn new() -> Typesetting {
        Typesetting {}
    }
}

/// A Page is made up of a number of boxes which span the page.
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Page {
    boxes: Vec<HorizontalBox>,
}

impl Page {
    fn new() -> Page {
        Page { boxes: vec![] }
    }

    fn render(&self, svg: &mut svg::Drawing) {
        let mut y: f32 = 0.0;
        for horizontal_box in self.boxes.iter() {

            horizontal_box.render(svg, y);

            y += horizontal_box.height();
        }
    }
}

/// A box that spans the page.
#[derive(Debug, PartialEq, PartialOrd, Clone)]
enum HorizontalBox {
    // TODO we may have multi-stave systems in future.
    System(Stave),
}

impl HorizontalBox {
    fn height(&self) -> f32 {
        match self {
            &HorizontalBox::System(ref stave) => stave.height() + SYSTEM_V_MARGIN,
        }
    }

    fn render(&self, svg: &mut svg::Drawing, y: f32) {
        match self {
            &HorizontalBox::System(ref stave) => stave.render(svg, y),
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
enum Glyph {
    SingleBar,
    DoubleBar,
    EndBar,
    OpenRepeat,
    CloseRepeat,
    /// Note head of (position-on-stave)
    /// If we're unable to determine the glyph, can be none.
    NoteHead(i32, Option<music::DurationGlyph>),
    Clef(music::Clef),
}

impl Glyph {}

/// Entity
#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
struct Entity {
    glyph: Glyph,
}

impl Entity {
    fn new(glyph: Glyph) -> Entity {
        Entity { glyph: glyph }
    }

    fn width(&self) -> f32 {
        match self.glyph {
            // TODO number of dots will make a difference.
            Glyph::NoteHead(_, glyph) => {
                // Space for the head.
                HEAD_WIDTH * 2.0 +
                    // Space for the dots.
                    match glyph {
                        Some(music::DurationGlyph { shape, dots }) => HEAD_WIDTH * dots as f32,
                        _ => 0.0,
                    }
            }

            // TODO add padding, but in a way that is flush with the end of the line.
            Glyph::SingleBar => 1.0,
            Glyph::DoubleBar => 5.0,
            Glyph::EndBar => 11.0,
            Glyph::OpenRepeat => 20.0,
            Glyph::CloseRepeat => 10.0,

            Glyph::Clef(_) => 50.0,
        }
    }

    fn render(&self, svg: &mut svg::Drawing, x: f32, y: f32) {
        match self.glyph {
            Glyph::Clef(clef) => {
                let yy = y + (LINES_IN_STAVE - clef.centre) as f32 * HEAD_HEIGHT;

                svg.rect(x, yy - HEAD_HEIGHT / 2.0, 10.0, HEAD_HEIGHT);
                svg.text(x, yy - HEAD_HEIGHT / 2.0, "clef".to_string());
            }
            Glyph::SingleBar => {
                svg.rect(
                    x,
                    y + HEAD_HEIGHT,
                    1.0,
                    (LINES_IN_STAVE - 1) as f32 * HEAD_HEIGHT,
                );
            }
            Glyph::DoubleBar => {
                svg.rect(
                    x,
                    y + HEAD_HEIGHT,
                    1.0,
                    (LINES_IN_STAVE - 1) as f32 * HEAD_HEIGHT,
                );
                svg.rect(
                    x + 5.0,
                    y + HEAD_HEIGHT,
                    1.0,
                    (LINES_IN_STAVE - 1) as f32 * HEAD_HEIGHT,
                );
            }
            Glyph::OpenRepeat => {
                svg.rect(
                    x,
                    y + HEAD_HEIGHT,
                    1.0,
                    (LINES_IN_STAVE - 1) as f32 * HEAD_HEIGHT,
                );

                let dot_size = 6.0;

                svg.rect_fill(
                    x + 5.0,
                    y + (LINES_IN_STAVE - 3) as f32 * HEAD_HEIGHT,
                    dot_size,
                    dot_size,
                );
                svg.rect_fill(
                    x + 5.0,
                    y + (LINES_IN_STAVE - 5) as f32 * HEAD_HEIGHT,
                    dot_size,
                    dot_size,
                );


            }
            Glyph::CloseRepeat => {
                let dot_size = 6.0;


                svg.rect_fill(
                    x,
                    y + (LINES_IN_STAVE - 3) as f32 * HEAD_HEIGHT,
                    dot_size,
                    dot_size,
                );
                svg.rect_fill(
                    x,
                    y + (LINES_IN_STAVE - 5) as f32 * HEAD_HEIGHT,
                    dot_size,
                    dot_size,
                );

                svg.rect(
                    x + 10.0,
                    y + HEAD_HEIGHT,
                    1.0,
                    (LINES_IN_STAVE - 1) as f32 * HEAD_HEIGHT,
                );
            }
            Glyph::EndBar => {

                svg.rect(
                    x,
                    y + HEAD_HEIGHT,
                    1.0,
                    (LINES_IN_STAVE - 1) as f32 * HEAD_HEIGHT,
                );
                svg.rect_fill(
                    x + 5.0,
                    y + HEAD_HEIGHT,
                    3.0,
                    (LINES_IN_STAVE - 1) as f32 * HEAD_HEIGHT,
                );
            }

            Glyph::NoteHead(position, glyph) => {
                let yy = (y + (LINES_IN_STAVE - position) as f32 * HEAD_HEIGHT);

                match glyph {
                    None => {
                        svg.text(x, yy, "?".to_string());
                    }
                    Some(music::DurationGlyph { shape, dots }) => {

                        // Note head
                        match shape {
                            music::DurationClass::Semibreve |
                            music::DurationClass::Minim => {
                                svg.rect(x, yy - HEAD_HEIGHT / 2.0, HEAD_WIDTH, HEAD_HEIGHT);
                            }

                            music::DurationClass::Crotchet |
                            music::DurationClass::Quaver |
                            music::DurationClass::Semiquaver |
                            music::DurationClass::Demisemiquaver => {
                                svg.rect_fill(x, yy - HEAD_HEIGHT / 2.0, HEAD_WIDTH, HEAD_HEIGHT);
                            }
                        }

                        // Stem
                        match shape {
                            music::DurationClass::Minim |
                            music::DurationClass::Crotchet |
                            music::DurationClass::Quaver |
                            music::DurationClass::Semiquaver |
                            music::DurationClass::Demisemiquaver => {
                                svg.rect(
                                    x + HEAD_HEIGHT * 1.5,
                                    yy + HEAD_HEIGHT / 2.0 - STEM_HEIGHT,
                                    1.0,
                                    STEM_HEIGHT,
                                );
                            }

                            _ => (),
                        }

                        // Tail 1
                        match shape {
                            music::DurationClass::Quaver |
                            music::DurationClass::Semiquaver |
                            music::DurationClass::Demisemiquaver => {
                                svg.rect_fill(
                                    x + HEAD_HEIGHT + 1.5,
                                    yy - HEAD_HEIGHT / 2.0 - STEM_HEIGHT,
                                    HEAD_HEIGHT * 2.0,
                                    2.0,
                                );
                            }

                            _ => (),
                        }

                        // Tail 2
                        match shape {
                            music::DurationClass::Semiquaver |
                            music::DurationClass::Demisemiquaver => {
                                svg.rect_fill(
                                    x + HEAD_HEIGHT + 1.5,
                                    yy - HEAD_HEIGHT / 2.0 - STEM_HEIGHT + HEAD_HEIGHT * 2.0,
                                    HEAD_HEIGHT * 2.0,
                                    2.0,
                                );
                            }

                            _ => (),
                        }

                        // Tail 3
                        match shape {
                            music::DurationClass::Demisemiquaver => {
                                svg.rect_fill(
                                    x + HEAD_HEIGHT + 1.5,
                                    yy - HEAD_HEIGHT / 2.0 - STEM_HEIGHT + HEAD_HEIGHT * 3.0,
                                    HEAD_HEIGHT * 2.0,
                                    2.0,
                                );
                            }

                            _ => (),
                        }

                        for dot in 0..dots {
                            svg.rect_fill(
                                x + HEAD_HEIGHT * 1.5 + (dot + 2) as f32 * HEAD_HEIGHT * 0.5,
                                yy - HEAD_HEIGHT / 2.0,
                                2.0,
                                2.0,
                            );
                        }
                    }

                    // svg.rect_fill(x, yy - HEAD_HEIGHT / 2.0, HEAD_HEIGHT * 1.5, HEAD_HEIGHT);
                }
            }
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
struct Stave {
    entities: Vec<Entity>,
}

impl Stave {
    fn new() -> Stave {
        Stave { entities: vec![] }
    }

    fn height(&self) -> f32 {
        // TODO Include size of stave, ledger lines, etc.
        // Currently this is 5 lines and spaces + one space either side.
        (HEAD_HEIGHT * LINES_IN_STAVE as f32) + STAVE_V_MARGIN
    }

    fn render(&self, svg: &mut svg::Drawing, y: f32) {
        let natural_width: f32 = self.entities.iter().map(|entity| entity.width()).sum();

        // TODO check divide by zero
        let scale = f32::min(STAVE_WIDTH / natural_width, MINIMUM_STAVE_SCALE);

        // TODO exclude the front-matter (clef, key, metre etc) and final bar from justification.

        let mut x = 0.0;
        for entity in self.entities.iter() {
            entity.render(svg, x * scale, y);
            x += entity.width();
        }

        let stave_width: f32 = natural_width * scale;

        for bar_i in 0..LINES_IN_STAVE {
            let yy = y + (LINES_IN_STAVE - bar_i) as f32 * HEAD_HEIGHT;

            // DEBUG: Draw stave positions.
            // svg.rect(x + bar_i as f32 * 5.0, yy, 2.0, 2.0);
            // svg.text(x + bar_i as f32 * 5.0, yy, format!("{}", bar_i));


            if bar_i % 2 == 0 {
                svg.rect(0.0, yy, stave_width, 1.0);

            }
        }

    }
}

pub fn typeset_from_ast(ast: tune_ast_three::Tune) -> Page {
    let mut page = Page::new();

    let mut current_stave = Stave::new();

    // Always have a key and time signature on the go.
    let mut key_signature = l::T::KeySignature(
        music::PitchClass {
            diatonic_pitch_class: music::DiatonicPitchClass::C,
            accidental: None,
        },
        music::Mode::Major,
    );
    let mut metre = music::Metre(4, 4);

    // TODO We only ever use treble clef at the moment.
    let mut current_clef = music::Clef::treble();

    for token in ast.prelude {
        match token {
            l::T::KeySignature(pitch_class, mode) => {
                key_signature = l::T::KeySignature(pitch_class, mode)
            }
            l::T::Metre(new_metre) => metre = new_metre,
            _ => (),
        }
    }

    current_stave.entities.push(
        Entity::new(Glyph::Clef(current_clef)),
    );
    // TODO add key signature with params
    // TODO add time signature with params.

    for voice in ast.voices {
        for token in voice {
            match token {
                l::T::Newline => {
                    page.boxes.push(HorizontalBox::System(current_stave));
                    current_stave = Stave::new();


                    current_stave.entities.push(
                        Entity::new(Glyph::Clef(current_clef)),
                    );
                    // TODO add key signature with params
                    // TODO add time signature with params.
                }

                // TODO can collapse some sequential things down into single glyphs.
                l::T::SingleBar => current_stave.entities.push(Entity::new(Glyph::SingleBar)),

                l::T::DoubleBar => current_stave.entities.push(Entity::new(Glyph::DoubleBar)),

                l::T::OpenRepeat => current_stave.entities.push(Entity::new(Glyph::OpenRepeat)),

                l::T::CloseRepeat => current_stave.entities.push(Entity::new(Glyph::CloseRepeat)),

                l::T::EndBar => current_stave.entities.push(Entity::new(Glyph::EndBar)),

                l::T::Note(note) => {
                    // TODO extras like accidentals etc.
                    let music::Note(pitch, duration) = note;
                    let clef_interval = current_clef.pitch.interval_to(pitch);

                    let position = (clef_interval.pitch_classes + current_clef.centre) as i32;
                    let glyph = duration.to_glyph();

                    current_stave.entities.push(Entity::new(
                        Glyph::NoteHead(position, glyph),
                    ));
                }

                _ => {
                    // Ignore
                    // TODO don't ignore!
                }
            }
        }
    }

    page.boxes.push(HorizontalBox::System(current_stave));

    page
}

pub fn render_page(page: Page) -> String {
    let mut svg = svg::Drawing::new();

    page.render(&mut svg);

    svg.render()
}
