use crate::read_level::{ExprList,Stmt,Expr,Const,Fun};
use std::collections::HashMap;

#[derive(Debug,Clone)]
pub struct NFun {
  pub params: Vec<String>
, pub body: usize
}
#[derive(Debug,Clone)]
pub enum NConst { I32(i32), F32(f32), F(NFun), Params(Vec<String>) }
#[derive(Debug,Clone)]
pub enum NValue { Var(String), C(NConst), Lst(Vec<usize>), App(usize), E(String)
                , Builtin(usize), Unfilled }

#[derive(Debug,Clone)]
pub struct ENode {
  pub rc: usize
, pub val: NValue
, pub args: Vec<usize>
}

#[derive(Debug,Clone)]
pub struct EArena {
  pub emptys: Vec<usize>
, pub vars: HashMap<String,usize>
, pub nodes: Vec<ENode>
}

pub const builtins: [fn(usize,&Vec<usize>,&mut EArena); 1] = [
  /*+*/ |out,args,ar: &mut EArena| { println!("{:?},{:?}{:?}",out,args,ar); match (&ar.nodes[args[0]].val,&ar.nodes[args[1]].val) {
          (NValue::C(NConst::I32(a)),NValue::C(NConst::I32(b))) => {
            ar.nodes[out] = new_enodev(NValue::C(NConst::I32(a+b))); },
          (NValue::C(NConst::F32(a)),NValue::C(NConst::F32(b))) => {
            ar.nodes[out] = new_enodev(NValue::C(NConst::F32(a+b))); },
          _ => { ar.nodes[out] = new_enodev(NValue::E("ERROR: type mismatch.".to_string())); } } }
];

pub fn new_earena() -> EArena {
  return EArena { emptys: vec![], vars: vec![].into_iter().collect(), nodes: vec![] };
}

pub fn new_earenan(n: ENode) -> EArena {
  return EArena { emptys: vec![], vars: vec![].into_iter().collect()
                , nodes: vec![n] };
}

pub fn new_enodev(val: NValue) -> ENode {
  return ENode { rc: 1, val: val, args: vec![] };
}

pub fn pnode(ar: &mut EArena, n: ENode) -> usize {
  match ar.emptys.pop() {
    Some(i) => { ar.nodes[i] = n; return i; },
    None => { ar.nodes.push(n); return ar.nodes.len()-1; } }
}

pub fn unroll_comma(e: &Expr, outv: &mut Vec<Expr>) {
  match e {
    Expr::App(f,args) => {
      if let Expr::Var(s) = f.as_ref() { if *s == ",".to_string() {
        assert!(args.len() == 2,"',' not dyadic?");
        outv.push((*args)[0].clone());
        unroll_comma(&(*args)[1],outv); } } },
    _ => { } }
}

pub fn rfc_2497(f: &Box<Expr>, args: &Box<Vec<Expr>>, ar: &mut EArena) -> usize {
  let pos = ar.nodes.len();
  let fe = expr_to_arena(f,ar);
  let g = (*args).iter().map(|ex| expr_to_arena(ex,ar)).collect();
  return pnode(ar,ENode { rc: 1, val: NValue::App(fe),
                          args: g }); }

pub fn expr_to_arena(e: &Expr, ar: &mut EArena) -> usize {
  match e {
    Expr::Var(s) => { return pnode(ar,new_enodev(NValue::Var(s.clone()))); },
    Expr::C(Const::I32(i)) => {
      return pnode(ar,new_enodev(NValue::C(NConst::I32(*i)))); },
    Expr::C(Const::F32(f)) => {
      return pnode(ar,new_enodev(NValue::C(NConst::F32(*f)))); }
    Expr::C(Const::Params(p)) => {
      return pnode(ar,new_enodev(NValue::C(NConst::Params(p.clone())))); }
    Expr::C(Const::F(Fun { params, body })) => {
      let pos = ar.nodes.len();
      let g = expr_to_arena(body.as_ref(),ar);
      return pnode(ar,ENode { rc: 1
                         , val: NValue::C(NConst::F(NFun { params: params.clone(),
                                  body: g }))
                         , args: vec![] }); },
    Expr::App(f,args) => {
      if let Expr::Var(s) = f.as_ref() { if *s == "@".to_string() {
        if (*args).len() == 2 {
          let fe = expr_to_arena(&(*args)[0],ar);
          let mut argsv = vec![];
          unroll_comma(&(*args)[1],&mut argsv);
          let g = argsv.iter().map(|ex| expr_to_arena(ex,ar)).collect();
          return pnode(ar,ENode { rc: 1, val: NValue::App(fe),
                                  args: g }); }
        else { return pnode(ar,new_enodev(NValue::E(format!("ERROR: non-dyadic '@' not implemented.")))); } } else { return rfc_2497(f,args,ar); } }
      else { return rfc_2497(f,args,ar); } },
    Expr::Lst(lst) => {
      if (*lst).len() == 1 {
        return expr_to_arena(&(*lst)[0],ar); }
      else {
        let pos = ar.nodes.len();
        let g = (*lst).iter().map(|ex| expr_to_arena(ex,ar)).collect();
        return pnode(ar,new_enodev(NValue::Lst(g))); } },
    Expr::E(s) => {
      return pnode(ar,new_enodev(NValue::E(s.clone()))); },
    q => {
      return pnode(ar,new_enodev(NValue::E(format!("ERROR: expr not recognized: {:?} .",q)))); } }
}

pub fn append_expr(ar: &mut EArena, br: &EArena, ins: usize) -> usize {
  let (ENode { rc: _, val: val, args: args }) = &br.nodes[ins];
  let pos = ar.nodes.len();
  let g = args.iter().map(|en| append_expr(ar,br,*en)).collect();
  return pnode(ar,ENode { rc: 1, val: val.clone(),
    args: g });
}

pub fn add_v(n: &String, ar: &EArena, v: usize, bvs: &mut HashMap<String,Vec<(EArena,usize)>>) {
  let mut z: EArena = new_earena();
  //let ef = append_expr(ar,br,v);
  match bvs.get_mut(n) {
    Some(q) => {
      (*q).push((z,v)); }
    None => { bvs.insert(n.clone(),vec![(z,v)]); } }
}

pub fn rem_v(n: &String, bvs: &mut HashMap<String,Vec<(EArena,usize)>>) {
  match bvs.get_mut(n) {
    Some(q) => {
      if q.len() <= 1 { bvs.remove(n); }
      else { (*q).pop(); } },
    None => { } }
}

pub fn adelete(q: usize, ar: &mut EArena) {
  if ar.nodes[q].rc <= 1 {
    ar.nodes[q] = new_enodev(NValue::Unfilled);
    ar.emptys.push(q);
    for z in ar.nodes[q].args.clone().iter() { adelete(*z,ar); } }
  else { ar.nodes[q].rc = ar.nodes[q].rc - 1; }
}

pub fn asubst(ins: usize, ar: &mut EArena, bvs: &mut HashMap<String,Vec<(EArena,usize)>>) {
  let (ENode { rc: _, val: v, args: args }) = ar.nodes[ins].clone();
  match v {
    NValue::Var(s) => {
      match ar.vars.get(&s) {
        Some(i) => { ar.nodes[*i].rc += 1; ar.nodes[ins] = ar.nodes[*i].clone(); },
        None => {
          if let Some((tr,i)) = bvs.get(&s).map(|x| &x[0]) {
            let nq = append_expr(ar,&tr,*i);
            ar.nodes[ins] = ar.nodes[nq].clone();
            ar.vars.insert(s.clone(),ins); } } } },
    NValue::C(NConst::F(NFun { params, body })) => {
      let mut r: Vec<(String,Vec<(EArena,usize)>)> = vec![];
        //params.iter().map(|p| bvs.remove_entry(p)).collect();
        
      for p in params.iter() { match bvs.remove_entry(p) { Some(q) => r.push(q), None => { } } }
      asubst(body,ar,bvs);
      for (n,v) in r { bvs.insert(n,v); } },
    NValue::App(fe) => {
      asubst(fe,ar,bvs);
      for a in args { asubst(a,ar,bvs); } },
    NValue::Lst(vs) => {
      for v in vs { asubst(v,ar,bvs); } },
    _ => { } }
}

//pub enum NValue { Var(String), C(NConst), Lst(Vec<usize>), App(usize), E(String), Unfilled }
pub fn aeval(ins: usize, ar: &mut EArena, bvs: &mut HashMap<String,Vec<(EArena,usize)>>) -> usize {
  let (ENode { rc: _, val: v, args: args }) = ar.nodes[ins].clone();
  match v {
    NValue::Var(s) => { // additive
      match bvs.get(&s) {
        Some(se) => { let g = append_expr(ar,&se.last().unwrap().0
                                            ,se.last().unwrap().1);
          ar.nodes[ins] = ar.nodes[g].clone();
          ar.nodes[g] = new_enodev(NValue::Unfilled); return ins; },
        None => { ar.nodes[ins] =
          new_enodev(NValue::E(format!("ERROR: variable '{:?}' not assigned",s)));
          return ins; } } },
    NValue::C(c) => { return ins; },
    NValue::Lst(lst) => { // not ground, recursive
      ar.nodes[ins] = new_enodev(NValue::Lst(lst.iter().map(|i| aeval(*i,ar,bvs)).collect()));
      return ins; },
    NValue::App(fe) => { // potentially subtractive
      let fz = aeval(fe,ar,bvs);
      let (ENode { rc: _, val: f, args: a }) = ar.nodes[fe].clone();
      let na: Vec<usize> = args.iter().map(|x| aeval(*x,ar,bvs)).collect(); // note 'args' here and
        // not 'a' !
      match ar.nodes[fz].val.clone() {
        NValue::C(NConst::F(NFun { params, body })) => {
          if na.len() == params.len() {
            for (p,n) in params.iter().zip(na.iter()) { add_v(p,ar,*n,bvs); }
            let b = asubst(body,ar,bvs);
            for p in params.iter() { rem_v(p,bvs); }
            for q in na { adelete(q,ar); }
            adelete(fe,ar);
            return /*b*/ ins; }
          else { ar.nodes[ins] = new_enodev(
            NValue::E(format!("ERROR: function has arity {:?} but only given are {:?} parameters.",params.len(),na.len())));
            adelete(fe,ar); return ins; } },
        NValue::Builtin(i) => {
          builtins[i](ins,&na,ar);
          for a in na { adelete(a,ar); }
          adelete(fe,ar); return ins; },
        q => { ar.nodes[ins] = new_enodev(NValue::E(format!("ERROR: '{:?}' is not a function.",fz)));
               return ins; } } },
    NValue::E(s) => { return ins; },
    NValue::Builtin(_) => { return ins; },
    NValue::Unfilled => { return ins; } }
}

/*pub const builtins: HashMap<String,Vec<Expr>> = [
  (",",|lst|
    if lst.len() == 2 {
      match (lst[0],lst[1]) {
        (l,Expr::Lst(r)) => {
          return (*r).clone().insert(0,l); },
        (l,r) => { return vec![l,r]; } } }
    else { return Error::E(format!("ERROR: operator ',' expects 2 arguments, {:?} given",lst.len())); },
  ("@",|lst|
    if lst.len() == 2 {
      match (eunpack(lst[0]),eunpack(lst[1])) {
        ([Expr::C(Const::F(Fun { params, body }))],[r]) => {*/        

/*pub fn eval(e: &Expr, bvs: &mut HashMap<String,Vec<Expr>>) -> Expr {
  match e {
    Expr::Var(s) => {
      match bvs.get(s) {
        Some(se) => { return se.clone(); }
        None => { return Expr::E(format!("ERROR: variable '{:?}' not assigned.",s)); } } },
    Expr::App(f,args) => {
      let ef = eval(f,bvs);
      let eargs = args.iter().map(|x| eval(x.clone(),bvs)).collect();
      match ef {
        Expr::Builtin(func) => {
          func(Box::new(eargs)); },
        Expr::C(Const::F(Fun { params, body })) => {
          if eargs.len() != params.len() {
            return Expr::E("ERROR: function has arity {:?} but only given is {:?} parameters."
                          ,params.len(),eargs.len()); }
          else {
            params.iter().zip(eargs.iter()).map(|(n,v)| add_v(*n,v.clone(),bvs));
            let es = subst(&*body,bvs);
            params.map(|n| rem_v(*n,bvs));
            return es; } }.
        Expr::E(s) => {
          return Error::E([s,format!("\n\t: function application in {:?} failed.",eformat(f))]
                          .concat()); } }, }*/
