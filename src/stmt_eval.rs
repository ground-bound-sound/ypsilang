use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::fs;
use std::collections::HashMap;
use crate::read_level::{Stmt,SErr,Expr,Const,Fun,stmtp,stmtsp};
use crate::eval_level::{NFun,NConst,NValue,ENode,EArena
                       ,new_earena,new_earenan,new_enodev,pnode,aeval,expr_to_arena
                       ,add_v};

#[derive(Debug,Clone)]
pub struct Module {
  pub name: String
, pub args: HashMap<String,(EArena,usize)>
//, pub vs: HashMap<String,Vec<(EArena,usize)>>
, pub fields: Vec<String>
}

#[derive(Debug,Clone)]
pub struct PState {
  pub bvs: HashMap<String,Vec<(EArena,usize)>>
, pub mods: HashMap<String,Module>
, pub errs: Vec<SErr>
}

pub fn new_pstate() -> PState {
  return PState { bvs: vec![].into_iter().collect()
                , mods: vec![].into_iter().collect()
                , errs: vec![] };
}

/*pub enum Stmt {
  VarDef(String,Expr), SetPlat(Expr), Import(Vec<String>), ModuleBegin(String,Vec<Expr>,Box<Vec<Stmt>>)
, Open(String), Type(Expr,Vec<Expr>), Ex(Expr), E(SErr)
}*/

pub fn seval(s: &Stmt, ps: &mut PState, m: &String, prec: &HashMap<String,usize>) -> Option<String> {
  match s {
    Stmt::VarDef(name,e) => {
      let mut ar = new_earena();
      let q = expr_to_arena(e,&mut ar);
      let g = aeval(q,&mut ar,&mut ps.bvs);
      add_v(&[m.clone(),name.clone()].concat(),&mut ar,g,&mut ps.bvs); // note that inserted into the variable list
        // is the name with all module context, whereas returned by 'seval' is the name alone.
      println!("{:?}",ps);
      return Some(name.clone()); },
    Stmt::SetPlat(e) => { return None; },
    Stmt::ModuleBegin(name,args,fs) => {
      match &ps.mods.get(name) {
        Some(_) => { return None; },
        None => {
          /*let mut ps0 = ps.clone();
          for f in fs.as_ref() { seval(f,ps0); }*/
          // NOTE: nested modules probably do not work as expected!
          let mut qs = vec![];
          let m2 = if *m == "".to_string() { [name.clone(),".".to_string()].concat() }
                   else { [m.clone(),name.clone()].concat() };
          let mut ps0 = ps.clone();
          for f in fs.as_ref() {
            if let Some(s) = seval(f,&mut ps0,&m2,prec) { qs.push(s.clone()); } }
          for q in &qs {
            let qg = [m2.clone(),q.clone()].concat();
            let el = ps0.bvs.get(&qg).unwrap().last().unwrap().clone();
            add_v(&qg,&el.0,el.1,&mut ps.bvs); }
          ps.mods.insert(name.clone(),Module { name: name.clone(), args: vec![].into_iter().collect()
                                             , fields: qs });
          return None; } } },
    Stmt::Import(names,args) => {
      /*let n2: &str= &name[..]
      let ms: Vec<&str> = n2.rsplitn(2,"/").collect();
      let (module,path) = (ms[0].to_string(),ms[1].to_string());*/
      /* should define new 'pstate' here and filter its 'bvs' for vars prefixed with 'm' and
       * merge that with 'ps.bvs' . */
      let module = names.last().unwrap();
      let path = (&names[0..names.len()-1]).join("/"); // needs to concat with '/' .
      match fs::read_to_string([path.clone(),".iy".to_string()].concat()) {
        Ok(s) => {
          let sts = stmtsp(&s,prec);
          match sts {
            Ok((_,stq)) => {
              for st in stq { /*seval(&st,ps,&[m.clone(),".".to_string(),module.clone()].concat(),prec);*/
                seval(&st,ps,m,prec); } },
            Err(e) => { println!("PARSE ERROR: {:?}",e); ps.errs.push(SErr::ParseError); } }
          return None; },
        Err(_) => {
          ps.errs.push(SErr::FileNotFound(path.clone())); } }
      return None; },
    Stmt::Open(name) => {
      match &ps.mods.get(name) {
        Some(Module { name: _, args: _, fields: fm }) => {
          for f in fm {
            let a = ps.bvs.get(&[m.clone(),name.clone(),".".to_string(),f.clone()]
                      .concat()).unwrap().last().unwrap();
            add_v(&f.clone(),&a.0.clone(),a.1.clone(),&mut ps.bvs);
            /* probably opens everywhere which is bad. */ }
          return None; },
        None => { ps.errs.push(SErr::ModuleNotFound(name.clone())); return None; } } },
    Stmt::Ex(e) => {
      let mut ar = new_earena();
      let q = expr_to_arena(&e,&mut ar);
      let g = aeval(q,&mut ar,&mut ps.bvs);
      println!("    {:?} {:?}",ar,g);
      return None; },
    Stmt::Type(e,fs) => { return None; },
    Stmt::E(s) => {
      println!("ERROR: {:?}",s);
      return None; } }
}
