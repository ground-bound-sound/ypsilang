extern crate nalgebra;
use nalgebra::{Vector2,Vector3};

use crate::player::{Player,Displacement,cross2d,new_pos,sgn,sgn_fpe_aware};

pub const BLKLEN: f32 = 0.1;

// must be quads!
// edges should be directed in clockwise order.
#[derive(Debug,Clone)]
pub struct Platform {
  pub tl: Vector2<f32>
, pub edges: Vec<Edge>
}

#[derive(Debug,Clone,PartialEq)]
pub enum EdgeFunc { Floor, Wall }

// NOTE: one block is 80x80 pix .
#[derive(Debug,Clone)]
pub struct Edge {
  pub pt1: Vector2<f32>
, pub pt2: Vector2<f32>
, pub func: EdgeFunc
}

// projection a onto b.
pub fn proj(a: Vector2<f32>, b: Vector2<f32>) -> Vector2<f32> {
  return b*a.dot(&b)/b.norm().powf(2.0);
}

pub fn proj_side(a: Vector2<f32>, b: Vector2<f32>) -> Vector2<f32> {
  return if cross2d(a,b) <= 0.0 { a } else { proj(a,b) };
}

// "line-line intersection" on wikipedia.
pub fn get_intersect(pl: &mut Player, d: &mut Displacement, np: Vector2<f32>
                    ,e: &Edge, tl: Vector2<f32>) -> Vector2<f32> {
  let pt1 = pl.pos;
  let pt2 = np;
  let pt3 = tl + e.pt1; // tl + ??
  let pt4 = tl + e.pt2; // tl + ??
  return 
    Vector2::new(((pt1.x*pt2.y-pt1.y*pt2.x)*(pt3.x-pt4.x)-(pt1.x-pt2.x)*(pt3.x*pt4.y-pt3.y*pt4.x))
                  /((pt1.x-pt2.x)*(pt3.y-pt4.y)-(pt1.y-pt2.y)*(pt3.x-pt4.x))
                ,((pt1.x*pt2.y-pt1.y*pt2.x)*(pt3.y-pt4.y)
                   -(pt1.y-pt2.y)*(pt3.x*pt4.y-pt3.y*pt4.x))
                  /((pt1.x-pt2.x)*(pt3.y-pt4.y)-(pt1.y-pt2.y)*(pt3.x-pt4.x)));
}

/*pub fn basic_intersect(pl: &mut Player, d: &mut Displacement, np: Vector2<f32>
                      ,e: &Edge, tl: Vector2<f32>) -> Vector2<f32> {
  let pt1 = pl.pos;
  let pt1 = np;
  let pt3 = tl+e.pt1;
  let pt4 = tl+e.pt2;*/

pub const SGN_ERR: f32 = 0.005;

pub fn obj_coll(obj: &Platform, oid: usize, pl: &mut Player, d: &mut Displacement, dt: f32)
  -> Option<(usize,Option<usize>)> {
  // for each edge: take cross product of before and after to see if sign is different.
  //   difference in sign tells which side the plane parallel to and on the edge the player is.
  let mut fin = None;
  let mut ein = true;
  let mut connected = false;
  let np = new_pos(pl,d,dt);
  for (i,Edge { pt1: p1, pt2: p2, func: f }) in obj.edges.iter().enumerate() {
    let ve = p2-p1;
    let vp0 = pl.pos-obj.tl-p1;  // DONE: added 'p1' to offset 've'.
    let vp1 = np-obj.tl-p1;

    // might need to make a floating-point error aware sgn function.
    // TODO: maybe use one-sided awareness.
    let q0 = sgn_fpe_aware(cross2d(vp0,ve),SGN_ERR);
    let q1 = sgn_fpe_aware(cross2d(vp1,ve),SGN_ERR);
    println!("{:?} {:?}",q0,q1);

    if /*q0 != q1*/ q0 == -1.0 && q1 == 1.0 
      || q0 == -1.0 && q1 == 0.0 || (q0 == 0.0 && q1 == 0.0) { fin = Some(i); }
    if q0 == 0.0 && q1 == 0.0 { connected = true; }
    //ein = ein && q0 != q1;
    ein = ein && (q1 == 1.0 || q1 == 0.0); // 1.0 is inside.
  }
  println!("inside? {:?}", ein);
  return if ein { 
    // NOTE the unwrapping of 'fin'.  In theory, the player should never be inside the object.
    // move player out of object and potentially set attached.
    if let Some(q) = fin { if !connected {
      pl.pos = get_intersect(pl,d,np,&obj.edges[q],obj.tl); } }
      //pl.pos = basic_intersect(pl,d,np,&obj.edges[q],obj.tl); }
    println!("intersection: {:?}",pl.pos);

    /*if obj.edges[fin.unwrap()].func == EdgeFunc::Floor {
      /*pl.attached = Some((oid,fin));*/ Some((oid,fin)) }
    else { Some((oid,fin)) } }*/
    Some((oid,fin)) }
  else { None };
}

pub fn test_collisions(objs: &Vec<Platform>, pl: &mut Player, d: &mut Displacement
                      ,dt: f32) -> Option<(usize,Option<usize>)> {
  if pl.attached.map_or(false,|(a,b)| a == 0) { return None; }
  else { 
    let q = obj_coll(&objs[0],0,pl,d,dt);
    return q; }
}
