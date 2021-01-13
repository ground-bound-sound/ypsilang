extern crate sdl2;
extern crate nalgebra;
use std::ffi::{CString, CStr};
use std::path::Path;
use nalgebra::base::{Vector2,Vector3};
use sdl2::rect::Rect;
use sdl2::keyboard::Scancode;
use sdl2::pixels::Color;
use sdl2::video::{WindowContext,Window};
use sdl2::surface::{Surface,SurfaceContext};
use sdl2::render::{TextureCreator,Texture,Canvas};
use std::str;
use std::io;
use std::write;
use std::io::Write;
use std::fs::{read_to_string};

pub mod player;
pub mod plat;
pub mod read_level;
use player::{Displacement};
use plat::{Platform,Edge,EdgeFunc};
use read_level::{levelp,exprp};
//pub mod entity;

fn create_jbox(texture: &Texture, canvas: &mut Canvas<Window>, r: Option<Rect>) -> String {
  canvas.copy(&texture,None,r); return "".to_string();
}

/*fn main() {
  let sdl = sdl2::init().unwrap();
  let video_subsystem = sdl.video().unwrap();
  let window = video_subsystem.window("rewriting",800,800)
    .position_centered().resizable().build().unwrap();

  let mut event_pump = sdl.event_pump().unwrap();
  let mut canvas = window.into_canvas().present_vsync().build().unwrap();
  canvas.clear();
  canvas.present();


  /*let mut plat: entity::Quad = entity::Quad
    { pos: Vector2::new(-1.0,0.0)
    , coords: (Vector2::new(0.0,0.0),Vector2::new(0.6,-0.3)
              ,Vector2::new(0.6,-0.5),Vector2::new(0.0,-0.3)) };
  let plats: Vec<entity::Quad> = vec!(plat.clone());
  let mut curve: entity::Curve = entity::Curve
    { ftype: entity::FType::Horizontal
    , ctype: entity::CType::Walkable
    , trans: entity::id_f
    , itrans: entity::id_f
    , f: |x| { (x-0.3).sin()/5.0-0.5 }
    , df: |x| { (x-0.3).cos()/5.0-0.5 }
    , disp: Vector2::new(0.0,0.0)
    , bound: (0.0,0.75) };
  let mut wall: entity::Curve = entity::Curve
    { ftype: entity::FType::Vertical
    , ctype: entity::CType::Wall
    , trans: entity::swap_axes_reflect
    , itrans: entity::swap_axes_reflect_inv
    , f: |y| { 0.0 }
    , df: |y| { 0.0 }
    , disp: Vector2::new(0.75,0.0)
    , bound: (-1.0,1.0) };
  let curves: Vec<entity::Curve> = vec!(curve.clone(),wall.clone());
  //let mut pos: Vector2<f32> = Vector2::new(0.0,0.0);*/
  
  let mut plats: Vec<Platform> = vec![Platform
    { tl: Vector2::new(0.0,-0.5)
    , edges: vec![
        Edge { pt1: Vector2::new(0.0,0.0), pt2: Vector2::new(0.5,0.0), func: EdgeFunc::Floor }
      , Edge { pt1: Vector2::new(0.5,0.0), pt2: Vector2::new(0.5,-0.5), func: EdgeFunc::Wall }
      , Edge { pt1: Vector2::new(0.5,-0.5), pt2: Vector2::new(0.0,-0.5), func: EdgeFunc::Wall }
      , Edge { pt1: Vector2::new(0.0,-0.5), pt2: Vector2::new(0.0,0.0), func: EdgeFunc::Wall }] }];
  let mut pl: player::Player = player::Player
    { pos: Vector2::new(0.3,0.0), vel: Vector2::new(0.0,0.0)
    , dir: Vector2::new(0.0,0.0), mass: 1.0
    , attached: None };

  let mut start: u64 = 0;
  let mut prev: u64 = 0; let mut now: u64 = 0;
  unsafe { now = sdl2::sys::SDL_GetPerformanceCounter(); }
  let mut dt: f32 = 0.0;
  let mut updp: Displacement = player::Displacement
    { dpdt: Vector2::new(0.0,0.0), dvdt: Vector2::new(0.0,0.0)
    , fs: Vector2::new(0.0,0.0) };

  let texture_creator = canvas.texture_creator();
  let s : Surface<'static> = Surface::load_bmp(Path::new("resources/jbox.bmp"))
      .unwrap();
  let texture = texture_creator.create_texture_from_surface(&s).unwrap();
  /*let m : Surface<'static> = Surface::load_bmp(Path::new("resources/map.bmp")).unwrap();
  let mt = texture_creator.create_texture_from_surface(&m).unwrap();*/

  'main: loop {
    prev = now;
    unsafe { now = sdl2::sys::SDL_GetPerformanceCounter(); }
    unsafe {
      dt = (((now-prev)*1000) as f32)/(sdl2::sys::SDL_GetPerformanceFrequency() as f32)*0.001; }
    canvas.set_draw_color(Color::RGB(150,230,90));
    canvas.clear();
    //create_jbox(&mt,&mut canvas,Some(Rect::new(0,0,800,800)));
    for event in event_pump.poll_iter() {
      match event { sdl2::event::Event::Quit  {..} => break 'main,
                    _ => {}, } }
    let keys = event_pump.keyboard_state();
    pl.dir = Vector2::new(0.0,0.0);
    // player update:
    if keys.is_scancode_pressed(Scancode::D) {
      pl.dir = Vector2::new(1.0,0.0);
      //pl.movement = pl.movement + pl.dir.x;
      player::update_p_move(&mut pl,&mut updp,dt); }
    if keys.is_scancode_pressed(Scancode::A) {
      pl.dir = Vector2::new(-1.0,0.0);
      //pl.movement = pl.movement + pl.dir.x;
      player::update_p_move(&mut pl,&mut updp,dt); }
    /*if keys.is_scancode_pressed(Scancode::W) {
      player::update_p_jump(&mut pl,dt); }*/
    player::update_p(&plats,&mut pl,&mut updp,dt);
    pl.dir = Vector2::new(0.0,0.0);

    create_jbox(&texture,&mut canvas,Some(
      Rect::new(((pl.pos.x-0.05)*800.0) as i32,((-pl.pos.y-0.05)*800.0) as i32
        ,(0.1*800.0) as u32,(0.1*800.0) as u32)));
    canvas.set_draw_color(Color::RGB(255,0,0));
    //canvas.fill_rect(Rect::new(((pl.pos.x)*800.0) as i32,(-pl.pos.y*800.0) as i32
    //  ,(0.1*800.0) as u32,(0.1*800.0) as u32));
    canvas.fill_rect(Rect::new((0.0*800.0) as i32,(-(-0.5)*800.0) as i32
                      ,(0.5*800.0) as u32,(0.5*800.0) as u32));
    canvas.present(); }
}*/

fn main() {
  loop {
    let prec = vec![(":".to_string(),0),("->".to_string(),100)
                   ,("@".to_string(),200),(",".to_string(),300)].into_iter().collect();
    let mut input = String::new();
    print!("> "); io::stdout().flush();
    match io::stdin().read_line(&mut input) {
      Ok(s) => {
        let (_,st) = exprp(&input,&prec).unwrap();
        println!("{:?}",st); },
      Err(e) => { println!("ERROR: {:?}",e); }
    }
  }
}
