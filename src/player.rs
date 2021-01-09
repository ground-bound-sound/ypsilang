extern crate nalgebra;
use nalgebra::base::{Vector2,Vector3};

use crate::plat::{Platform,Edge,test_collisions,proj,proj_side};

pub const GRAV: f32 = 1.0;
pub const FRIC: f32 = 0.5;

pub struct Player {
  pub pos: Vector2<f32>
, pub dir: Vector2<f32>
, pub vel: Vector2<f32>
, pub mass: f32
, pub attached: Option<(usize,Option<usize>)>
}

pub struct Displacement {
// all d* variables represent d*/dt .  TODO: fix?
  pub dpdt: Vector2<f32>
, pub dvdt: Vector2<f32>
, pub fs: Vector2<f32>
}

pub fn sgn(x: f32) -> f32 {
  return if x > 0.0 { 1.0 } else if x < 0.0 { -1.0 } else { 0.0 };
}

pub fn sgn_fpe_aware(x: f32, err: f32) -> f32 {
  return if x < err && x > -err { 0.0 } else if x >= err { 1.0 } else { -1.0 };
}

pub fn cross2d(a: Vector2<f32>, b: Vector2<f32>) -> f32 {
  return a.x*b.y-b.x*a.y;
}

pub fn f_fric(m: f32, dir: Vector2<f32>) -> Vector2<f32> {
  return -FRIC*GRAV*m*dir; }
pub fn f_grav(m: f32) -> Vector2<f32> { return Vector2::new(0.0,-GRAV*m); }

/*pub fn update_p_move(pl: &mut Player, d: &mut Displacement, dt: f32) {
  d.dv.x = d.dv.x + pl.dir.x;
}*/

// dir.x is expected to be nonnegative.
pub fn update_p_fric(dir: Vector2<f32>, pl: &mut Player, d: &mut Displacement, dt: f32) {
  let vel = pl.vel + (d.dvdt+d.fs/pl.mass)*dt;
  if vel != Vector2::new(0.0,0.0) {
    let s = sgn(cross2d(vel,dir));
    println!("{:?}, vel: {:?}",vel.normalize(),vel);
    let fr = f_fric(pl.mass,vel.normalize());
    let z = vel - fr/pl.mass*dt;
    /*if sgn(z) != s {
      d.dv =  } else { fr }*/
    d.fs = d.fs + fr; }
}

pub fn update_p_grav(pl: &mut Player, d: &mut Displacement, dt: f32) {
  d.fs = d.fs + f_grav(pl.mass);
}
pub fn update_p_nograv(pl: &mut Player, d: &mut Displacement, dt: f32) {
  // probably not needed!
}

pub fn update_p_move(pl: &mut Player, d: &mut Displacement, dt: f32) {
  d.dvdt.x = d.dvdt.x + pl.dir.x;
}

pub fn update_p(objs: &Vec<Platform>, pl: &mut Player, d: &mut Displacement, dt: f32) {

/*  let s = sgn(pl.vel.x);
  let z = pl.vel.x+d.dp.x - f_fric(pl.mass)/pl.mass*dt;
  println!("{:?}",z);
  let fricx = if sgn(z) != s { 0.0 } else { z };

  pl.vel = pl.vel + Vector2::new(d.dv.x-fricx,d.dv.y)*dt;
  pl.pos = pl.pos + pl.vel*dt;

  d.dp = Vector2::new(0.0,0.0);
  d.dv = Vector2::new(0.0,0.0);*/

  /*match pl.attached {
    Some((oid,Some(surf))) => {
      let (Platform { tl: tl, edges: edges }) = &objs[oid];
      update_p_fric(Vector2::new(1.0,0.0),pl,d,dt);

      let q = &objs[oid].edges[surf];
      let v = q.pt2-q.pt1;
      d.dpdt = proj(d.dpdt,v);
      d.dvdt = proj(d.dvdt,v);
      d.fs = proj(d.fs,v); },
    Some((_,_)) => { },
    None => {
      update_p_grav(pl,d,dt); } }*/

  // for walking:
  //   if total force is low enough, the player stops entirely. (represents friction)
  //   if y-component of force is low enough, the player can (attempt to) walk.

  update_p_grav(pl,d,dt);

  match test_collisions(objs,pl,d,dt) {
    Some((oid,Some(surf))) => { // player is already pushed out of object.
      let (Platform { tl: tl, edges: edges }) = &objs[oid];

      let q = &objs[oid].edges[surf];
      let v = q.pt2-q.pt1;
      d.dpdt = proj_side(d.dpdt,v);
      d.dvdt = proj_side(d.dvdt,v);
      d.fs = proj_side(d.fs,v);
      pl.vel = proj_side(pl.vel,v);
      println!("dp: {:?}, fs: {:?}",d.dvdt,d.fs);
      update_p_fric(Vector2::new(1.0,0.0),pl,d,dt); },
    Some((oid,None)) => { },
    None => { }
  }

  pl.vel = pl.vel + (d.dvdt + d.fs/pl.mass)*dt;
  pl.pos = pl.pos + pl.vel*dt;
  println!("pvel: {:?}, ppos: {:?}, dvdt: {:?}, dfs: {:?} ",pl.vel,pl.pos,d.dvdt,d.fs);

  d.dpdt = Vector2::new(0.0,0.0);
  d.dvdt = Vector2::new(0.0,0.0);
  d.fs = Vector2::new(0.0,0.0);
}

pub fn new_pos(pl: &Player, d: &Displacement, dt: f32) -> Vector2<f32> {
  let vel0 = pl.vel + (d.dvdt + d.fs/pl.mass)*dt;
  return pl.pos + vel0*dt;
}
