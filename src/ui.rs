use crate::chess::{generate_moves, postprocess_move, GameData, PieceColor, PieceType, Position};
use crate::graphics::{Drawable, Rect, Shader, ShaderProgram, Sprite, Texture2D};
use nalgebra_glm as glm;
use sdl2::{self, event::Event, mouse::MouseButton};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

const FPS: u64 = 60;
const FRAME_DURATION: Duration = Duration::from_millis(1000 / FPS);

pub fn run() {
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let gl_attr = video_subsystem.gl_attr();

    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);

    let window = video_subsystem
        .window("Chess2D", 768, 768)
        .opengl()
        .build()
        .unwrap();
    let _gl_context = window.gl_create_context().unwrap();
    let _gl =
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);
    let projection = &glm::ortho::<f32>(0.0, 764.0, 0.0, 764.0, -1.0, 1.0);

    unsafe {
        gl::Viewport(
            0,
            0,
            window.size().0.try_into().unwrap(),
            window.size().1.try_into().unwrap(),
        );
        gl::ClearColor(0.3, 0.3, 0.5, 1.0);
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }
    let texture_pack = match stb_image::image::load("./resources/textures/spritesheet.png") {
        stb_image::image::LoadResult::ImageU8(img) => Rc::new(img),
        _ => panic!("unsupported image"),
    };
    let (board_program, piece_program) = init_shaders();
    let texture = Rc::new(Texture2D::new(texture_pack.clone(), gl::RGBA));
    let piece_texture_map = create_piece_texture_map();
    let mut board = Rect::new(
        glm::vec4::<f32>(0.0, 0.0, window.size().0 as f32, window.size().1 as f32),
        board_program.clone(),
    );
    board.uniform_setter = Some(Box::new(|shader: Rc<ShaderProgram>| {
        shader.set_uniform_bool("black_view", false);
        shader.set_uniform_vec3f("white_color", glm::vec3(0.98, 0.96, 0.89));
        shader.set_uniform_vec3f("black_color", glm::vec3(1.0, 0.38, 0.38));
        shader.set_uniform_float("opacity", 1.0);
        shader.set_uniform_int("side_size", 96);
    }));
    let mut game_data = GameData::default();
    let mut valid_moves = generate_moves(&game_data);
    let mut selected = None;
    let mut to_be_promoted: Option<Position> = None;
    let mut selected_pos = glm::vec2::<f32>(0.0, 0.0);
    let mut event_pump = sdl.event_pump().unwrap();
    let mut last_frame_time = Instant::now();

    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,
                Event::MouseButtonDown {
                    timestamp: _,
                    window_id: _,
                    which: _,
                    mouse_btn,
                    clicks,
                    x,
                    y,
                } => {
                    if to_be_promoted.is_some() {
                        let pos = Position {
                            x: (x / 48) as i8,
                            y: (y / 48) as i8,
                        };
                        if pos.x != 0 || !(6..10).contains(&pos.y) {
                            continue;
                        }
                        game_data.board.remove(&to_be_promoted.unwrap());
                        let opposite = game_data.to_move.get_opposite();
                        game_data.board.insert(
                            to_be_promoted.unwrap(),
                            match pos.y {
                                6 => PieceType::Queen(opposite),
                                7 => PieceType::Rook(opposite),
                                8 => PieceType::Knight(opposite),
                                9 => PieceType::Bishop(opposite),
                                _ => panic!("cant happen"),
                            },
                        );
                        valid_moves = generate_moves(&game_data);
                        if valid_moves.is_empty() {
                            println!("the end; winner is {:?}", game_data.to_move.get_opposite());
                            break 'main;
                        }
                        println!("{game_data}");
                        for (pos, avail) in valid_moves.iter() {
                            println!("{pos:?} [{avail:?}]");
                        }
                        to_be_promoted = None;
                        continue;
                    }
                    let pos = Position {
                        x: (x / 96) as i8,
                        y: 7 - (y / 96) as i8,
                    };
                    if let Some(start_pos) = selected {
                        if valid_moves
                            .get(&start_pos)
                            .and_then(|valid_positions| Some(valid_positions.contains(&pos)))
                            .unwrap_or(false)
                        {
                            (game_data, to_be_promoted) =
                                postprocess_move(&game_data, start_pos, pos);
                            if to_be_promoted.is_some() {
                                selected = None;
                                continue;
                            }
                            valid_moves = generate_moves(&game_data);
                            if valid_moves.is_empty() {
                                println!(
                                    "the end; winner is {:?}",
                                    game_data.to_move.get_opposite()
                                );
                                break 'main;
                            }
                            println!("{game_data}");
                            for (pos, avail) in valid_moves.iter() {
                                println!("{pos:?} [{avail:?}]");
                            }
                        } else {
                            println!("cant go from {:?} to {:?}", start_pos, pos);
                        }
                    }
                    if clicks % 2 == 0 || mouse_btn != MouseButton::Left {
                        selected = None;
                        continue;
                    }
                    if let Some(&piece) = game_data.board.get(&pos) {
                        if piece.get_color() != game_data.to_move {
                            selected = None;
                            continue;
                        }
                    } else {
                        selected = None;
                        continue;
                    }
                    //if already selected => make move and switch game data / promotions
                    selected = match selected {
                        None => Some(pos),
                        Some(_) => None,
                    };
                    selected_pos = glm::vec2(x as f32 - 48.0, 768.0 - y as f32 - 48.0);
                    println!("Selected pos {:?}", selected);
                }
                Event::MouseMotion {
                    timestamp: _,
                    window_id: _,
                    which: _,
                    mousestate: _,
                    x,
                    y,
                    xrel: _,
                    yrel: _,
                } => {
                    if selected.is_none() {
                        continue;
                    }
                    selected_pos = glm::vec2(x as f32 - 48.0, 768.0 - y as f32 - 48.0);
                }
                _ => {}
            }
        }
        unsafe {
            gl::ClearColor(0.3, 0.3, 0.5, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        board.draw(&projection);
        draw(
            &game_data,
            selected,
            piece_program.clone(),
            &piece_texture_map,
            texture.clone(),
            &projection,
        );
        if selected.is_some() {
            Sprite::new(
                piece_program.clone(),
                texture.clone(),
                *piece_texture_map
                    .get(game_data.board.get(&selected.unwrap()).unwrap())
                    .unwrap(),
                glm::vec4::<f32>(selected_pos.x, selected_pos.y, 96.0, 96.0),
            )
            .draw(projection);
        }
        if to_be_promoted.is_some() {
            let opposite = game_data.to_move.get_opposite();
            Sprite::new(
                piece_program.clone(),
                texture.clone(),
                *piece_texture_map.get(&PieceType::Bishop(opposite)).unwrap(),
                glm::vec4::<f32>(0.0, 96.0 * 3.0, 48.0, 48.0),
            )
            .draw(projection);
            Sprite::new(
                piece_program.clone(),
                texture.clone(),
                *piece_texture_map
                    .get(&&PieceType::Knight(opposite))
                    .unwrap(),
                glm::vec4::<f32>(0.0, 96.0 * 3.5, 48.0, 48.0),
            )
            .draw(projection);
            Sprite::new(
                piece_program.clone(),
                texture.clone(),
                *piece_texture_map.get(&&&PieceType::Rook(opposite)).unwrap(),
                glm::vec4::<f32>(0.0, 96.0 * 4.0, 48.0, 48.0),
            )
            .draw(projection);
            Sprite::new(
                piece_program.clone(),
                texture.clone(),
                *piece_texture_map
                    .get(&&&&PieceType::Queen(opposite))
                    .unwrap(),
                glm::vec4::<f32>(0.0, 96.0 * 4.5, 48.0, 48.0),
            )
            .draw(projection);
        }
        window.gl_swap_window();
        // fps
        let frame_time = last_frame_time.elapsed();
        if frame_time < FRAME_DURATION {
            std::thread::sleep(FRAME_DURATION - frame_time);
        }
        // Update last_frame_time to measure the next frame's duration
        last_frame_time = Instant::now();
    }
}

fn draw(
    game_data: &GameData,
    selected: Option<Position>,
    piece_program: Rc<ShaderProgram>,
    piece_texture_map: &HashMap<PieceType, glm::Vec4>,
    texture: Rc<Texture2D>,
    projection: &glm::Mat4,
) {
    for (&p_pos, &p_type) in game_data.board.iter() {
        if selected.is_some() && selected.unwrap() == p_pos {
            continue;
        }
        Sprite::new(
            piece_program.clone(),
            texture.clone(),
            *piece_texture_map.get(&p_type).unwrap(),
            glm::vec4::<f32>(p_pos.x as f32 * 96.0, p_pos.y as f32 * 96.0, 96.0, 96.0),
        )
        .draw(projection);
    }
}
fn init_shaders() -> (Rc<ShaderProgram>, Rc<ShaderProgram>) {
    let board_vert =
        Shader::from_file("./resources/shaders/simple.v.glsl", gl::VERTEX_SHADER).unwrap();
    let board_frag =
        Shader::from_file("./resources/shaders/board.f.glsl", gl::FRAGMENT_SHADER).unwrap();
    let texture_vert =
        Shader::from_file("./resources/shaders/texture.v.glsl", gl::VERTEX_SHADER).unwrap();
    let texture_frag =
        Shader::from_file("./resources/shaders/texture.f.glsl", gl::FRAGMENT_SHADER).unwrap();

    let mut board_program = ShaderProgram::from_shaders(&[board_vert, board_frag]).unwrap();
    board_program.hash_uniform_locations(&[
        "black_view",
        "opacity",
        "side_size",
        "black_color",
        "white_color",
        "mvp",
    ]);
    let mut piece_program = ShaderProgram::from_shaders(&[texture_vert, texture_frag]).unwrap();
    piece_program.hash_uniform_locations(&["mvp"]);
    (board_program.into(), piece_program.into())
}
fn create_piece_texture_map() -> HashMap<PieceType, glm::Vec4> {
    let mut textures = HashMap::<PieceType, glm::Vec4>::new();
    generate_textures_for_side(0.0, PieceColor::Black, &mut textures);
    generate_textures_for_side(480.0, PieceColor::White, &mut textures);
    textures
}
fn generate_textures_for_side(
    y: f32,
    color: PieceColor,
    textures: &mut HashMap<PieceType, glm::Vec4>,
) {
    textures.insert(
        PieceType::Bishop(color),
        glm::vec4::<f32>(0.0, y, 480.0, 480.0),
    );
    textures.insert(
        PieceType::King(color),
        glm::vec4::<f32>(480.0, y, 480.0, 480.0),
    );
    textures.insert(
        PieceType::Knight(color),
        glm::vec4::<f32>(2.0 * 480.0, y, 480.0, 480.0),
    );
    textures.insert(
        PieceType::Pawn(color),
        glm::vec4::<f32>(3.0 * 480.0, y, 480.0, 480.0),
    );
    textures.insert(
        PieceType::Queen(color),
        glm::vec4::<f32>(4.0 * 480.0, y, 480.0, 480.0),
    );
    textures.insert(
        PieceType::Rook(color),
        glm::vec4::<f32>(5.0 * 480.0, y, 480.0, 480.0),
    );
}
