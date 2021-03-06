extern crate nom;
use nom::{
  IResult, Err, combinator::{opt,not,peek,flat_map,map,map_parser}, tuple
, bytes::complete::{tag,take_while}
, character::complete::{digit0,digit1,multispace0,multispace1,one_of,none_of,alphanumeric1}
, multi::{many0,many1,separated_list0,separated_list1}, sequence::delimited, branch::alt
, error::{ParseError,Error,ErrorKind,make_error}
};
use std::str;
use std::io;
use std::io::Write;
use std::collections::VecDeque;
use std::collections::HashMap;

use crate::eval_level::{EArena};

#[derive(Debug,Clone)]
pub struct ExprList {
  stmts: Vec<Expr>
}

#[derive(Debug,Clone)]
pub enum SErr { TypeMismatch((EArena,usize),(EArena,usize)), FileNotFound(String)
              , ParseError, ModuleNotFound(String) }

#[derive(Debug,Clone)]
pub enum Stmt {
  VarDef(String,Expr), SetPlat(Expr), Import(Vec<String>,Vec<Expr>), ModuleBegin(String,Vec<Expr>,Box<Vec<Stmt>>)
, Open(String), Type(Expr,Vec<Expr>), Ex(Expr), E(SErr)
}

#[derive(Debug,Clone,PartialEq)]
pub enum Expr {
  Var(String), C(Const), App(Box<Expr>,Box<Vec<Expr>>), Lst(Box<Vec<Expr>>), E(String)
, LParen, RParen, LP(i32), RP(i32), MetaMkOpr, Builtin(String)
, NoOp
}

#[derive(Debug,Clone,PartialEq)]
pub enum Const { I32(i32), F32(f32), F(Fun), Params(Vec<String>) }

#[derive(Debug,Clone,PartialEq)]
pub struct Fun {
  pub params: Vec<String>
, pub body: Box<Expr>
}

fn is_digit(c: char) -> bool {
  return c.is_digit(10);
}

fn intp(input: &str) -> IResult<&str,i32> {
  let (input,c) = opt(tag("-"))(input)?;
  let sgn = if c == Some("-") { "-" } else { "" };
  let (input,a) = digit1(input)?;
  return Ok((input,[sgn,a].concat().parse::<i32>().unwrap()));
}

fn floatp(input: &str) -> IResult<&str,f32> {
  let (input,c) = opt(tag("-"))(input)?;
  let sgn = if c == Some("-") { "-" } else { "" };
  let (input,a) = digit1(input)?;
  let (input,_) = tag(".")(input)?;
  let (input,b) = digit1(input)?;
  return Ok((input,[sgn,a,b].concat().parse::<f32>().unwrap()));
}

fn fnp<'a>(input: &'a str, prec: &HashMap<String,usize>) -> IResult<&'a str,Fun> {
  let (input,vars) = delimited(tag("{"),many0(delimited(multispace0,varp,multispace0)),tag("}"))
                       (input)?;
  let (input,_) = multispace0(input)?;
  let (input,body) = exprp(input,prec)?;
  return Ok((input,Fun { params: vars, body: Box::new(body) }));
}

fn paramsp<'a>(input: &'a str, prec: &HashMap<String,usize>) -> IResult<&'a str,Const> {
  let (input,vars) = delimited(tag("{"),many0(delimited(multispace0,varp,multispace0)),tag("}"))
                       (input)?;
  return Ok((input,Const::Params(vars)));
}

fn constp<'a>(input: &'a str, prec: &HashMap<String,usize>) -> IResult<&'a str,Const> {
  return alt((map(intp,|x| Const::I32(x)),map(floatp,|x| Const::F32(x))
             ,|x| paramsp(x,prec)))(input);
}

fn varp(input: &str) -> IResult<&str,String> {
  let (input,q) = alt((map(tag(","),|x: &str| x.to_string())
                      ,map(many1(none_of(" \r\n{}[]();"))
                          ,|x| x.into_iter().collect::<String>())))(input)?;
  return Ok((input,q));
}

fn lstp<'a>(input: &'a str, prec: &HashMap<String,usize>) -> IResult<&'a str,Vec<Expr>> {
  return separated_list1(multispace1,map(|x| constp(x,prec),|x| Expr::C(x)))(input);
}

/*fn argp<'a>(input: &'a str, prec: &HashMap<String,usize>) -> IResult<&'a str,Expr> {
  let (input,_) = multispace0(input)?;
  return alt((map(|x| lstp(x,prec),|x| Expr::Lst(Box::new(x)))
             ,delimited(tag("("),|x| exprp(x,prec),tag(")"))))(input);
}*/

fn lparen(input: &str) -> IResult<&str,Expr> {
  let (input,_) = delimited(multispace0,tag("("),multispace0)(input)?;
  return Ok((input,Expr::LParen));
}

fn rparen(input: &str) -> IResult<&str,Expr> {
  let (input,_) = delimited(multispace0,tag(")"),multispace0)(input)?;
  return Ok((input,Expr::RParen));
}

fn argp<'a>(input: &'a str, prec: &HashMap<String,usize>) -> IResult<&'a str,Expr> {
  let (input,_) = multispace0(input)?;
  return alt((lparen,rparen,map(|x| lstp(x,prec),|x| Expr::Lst(Box::new(x)))
             ,map(varp,|x| Expr::Var(x))))(input);
}

fn oprp<'a>(input: &'a str, prec: &HashMap<String,usize>) -> IResult<&'a str,Expr> {
  let (input,_) = multispace0(input)?;
  return alt((delimited(tag("("),|x| exprp(x,prec),tag(")")),map(varp,|x| Expr::Var(x))))(input);
}

/*fn apporlstp(input: &str) -> IResult<&str,Expr> {
  let (input,head) = argp(input)?;
  match peek(map_parser(multispace0,tag("@")))(input) {
    Ok((input,_)) => {
      let (input,_) = map_parser(multispace0,tag("@"))(input)?;
      let (input,ne) = exprp(input)?;
      return Ok((input,Expr::App(Box::new(head),Box::new(vec![lst])))); },
    Err(Err::Error((input,_))) => {
      let mut v = vec![head];

      /*while let Ok((_,_)) = peek(map_parser(multispace0,none_of(";")))(input) {
        match peek(map_parser(multispace0,tag("~")))(input) {
          Ok((input,_)) => {
            let (input,_) = tag("~")(input)?;
            let (input,a) = argp(input)?;
            let (input,_) = multispace0(input)?;
            let (input,n) = exprp(input)?;
            return Ok((input,Expr::App(Box::new(a),Box::new(vec![Expr::Lst(Box::new(v)),n])))); },
          Err(Err::Error((input,_))) => {
            let (input,a) = argp(input)?;
            v.push(a); }
          _ => { return map(tag("oops"),|_| Expr::E("oops".to_string()))(input); } } }*/
    },
      return Ok((input,Expr::Lst(Box::new(v)))); }
    _ => { return map(tag("oops"),|_| Expr::E("oops".to_string()))(input); } }
}*/

/*fn apply_f(z: &Expr, outq: &mut VecDeque<Expr>) {
  let sa = outq.pop_front(); let sb = outq.pop_front();
  match (sa,sb) {
    (Some(a),Some(b)) => {
      outq.push_back(Expr::App(Box::new(z.clone()),Box::new(vec![a,b]))); },
    _ => { outq.push_back(Expr::E("ERROR: expression parse error.".to_string())); }
  }
}*/

#[derive(Debug,Clone)]
enum Value { Opr(Expr), Val(Expr) }
impl Value {
  pub fn unwrap(&self) -> &Expr {
    match self { Value::Opr(e) => e, Value::Val(e) => e } }
}

fn apply_fs(outq: &mut VecDeque<Value>, ops: &Vec<(Expr,usize)>) -> Vec<Expr> {
  println!("({:?},{:?})",outq,ops);
  outq.append(&mut ops.iter().rev().map(|(a,_)| Value::Opr(a.clone())).collect::<VecDeque<Value>>());
  /*return outq.iter().fold(vec![]
           ,|n,val| match val {
              Value::Opr(o) => {
                let sa = n.pop(); let sb = n.pop();
                if let (Some(a),Some(b)) = (sa,sb) {
                  n.push(Expr::App(Box::new(o.clone()),Box::new(vec![b.clone(),a.clone()]))); }
                else { n.push(Expr::E("ERROR: syntax error caused stack underflow.".to_string())); }
              },
              Value::Val(v) => { n.push(v.clone()); } });*/
  let mut n : Vec<Expr> = vec![];
  for val in outq.iter() {
    match val {
      /*Value::Opr(Expr::MetaMkOpr) => {
        let so = n.pop(); let sa = n.pop();
        if let (Some(o),Some(a)) = (so,sa) {
          n.push(Expr::App(Box::new(o.clone()),Box::new(vec![a.clone()]))); }
        else { n.push(Expr::E("ERROR: syntax error caused stack underflow".to_string())); } },*/
      Value::Opr(o) => {
        let sa = n.pop(); let sb = n.pop();
        if let (Some(a),Some(b)) = (sa,sb) {
          n.push(Expr::App(Box::new(o.clone()),Box::new(vec![b.clone(),a.clone()]))); }
        else { n.push(Expr::E("ERROR: syntax error caused stack underflow.".to_string())); } },
      Value::Val(v) => { n.push(v.clone()); } }
  }
  return n;
}


fn add_f(z: &Expr, outq: &mut VecDeque<Value>) {
  outq.push_back(Value::Opr(z.clone()));
}

fn stmtendp(input: &str) -> IResult<&str,String> {
  let (input,_) = multispace0(input)?;
  println!("here: {:?}",input);
  return map(one_of(";}"),|x: char| vec![x].into_iter().collect())(input);
}

fn toggle(a: i32) -> i32 { return if a == 0 { 1 } else { 0 }; }

macro_rules! outqr {
  ($inp:expr,$outq:expr,$ops:expr) => { {
    let a = apply_fs(&mut $outq,&$ops);
    return match a.get(0) {
      Some(c) => Ok(($inp,c.clone())),
      None => map(tag("iqojeioqhworwh"),|x| Expr::E("j".to_string()))($inp) }; } }
}
fn apporlstp<'a>(input: &'a str, prec: &HashMap<String,usize>) -> IResult<&'a str,Expr> {
  let mut outq : VecDeque<Value> = VecDeque::new();
  let mut ops : Vec<(Expr,usize)> = vec![];
  let mut inp = input;
  let mut form = 0; // 0: Val, 1: Opr .
  while let Ok((input,c)) = peek(not(stmtendp))(inp) {
    //let (input,l) = argp(input,prec)?; // the error is that 'argp' fails.
    let i = argp(input,prec);
    println!("i: {:?}, (outq: {:?}, ops: {:?})",i,outq,ops);

    match i {
      Ok((input,Expr::LParen)) => {
        ops.push((Expr::LP(form),0));
        inp = input; form = 0; },
      Ok((input,Expr::RParen)) => {
        let mut stk: Vec<(Expr,usize)> = vec![];
        while ops.last().unwrap().0 != Expr::LP(0)
           && ops.last().unwrap().0 != Expr::LP(1) {
          /*stk.push_back(Value::Opr(ops.pop().unwrap().0));*/
          stk.push(ops.pop().unwrap()); }
        //stk.reverse(); // note the change here!
        match ops.pop().unwrap().0 {
          Expr::LP(0) => {
            outq.append(&mut stk.into_iter().map(|x| Value::Opr(x.0)).collect());
            inp = input; form = 1; },
          Expr::LP(1) => {
            //ops.push((apply_fs(&mut stk,&vec![])[0].clone(),0));
            /*outq.append(&mut stk);
            outq.push_back(Value::Opr(Expr::MetaMkOpr));*/
            // this method depends on EVERY operator having arity 2 .
            /*ops.push((apply_fs(&mut apply_fs(
                                &mut outq,&vec![]).map(|x| Value::Val(x))
                                .append(&mut stk),&vec![]).last().unwrap().clone(),0));*/
            outq = apply_fs(&mut outq,&stk).into_iter().map(|x| Value::Val(x)).collect();
            ops.push((outq.pop_back().unwrap().unwrap().clone(),0));

            inp = input; form = 0; },
          _ => { panic!("this should never happen!"); } } },
      Ok((input,q)) => {
        if form == 0 { outq.push_back(Value::Val(q)); inp = input; form = 1; }
        else { match &q {
          Expr::Var(s) => {
            match prec.get(s) {
              Some(i) => {
                if *i < ops.last().map(|x| x.1).unwrap_or(0) {
                  add_f(&Expr::Var(s.clone()),&mut outq);
                  ops.pop();
                  inp = input; form = 0; }
                else { ops.push((Expr::Var(s.clone()),*i));
                  inp = input; form = 0; } },
              None => {
                ops.push((q.clone(),usize::MAX)); inp = input; form = 0; } } },
          z => { panic!("parse error (2)!"); } } } },
      Err(e) => { return Err(e); }
      e => { return outqr!(inp,outq,ops); } } }
  return outqr!(inp,outq,ops); // maybe 'input' is weird?
}

macro_rules! stag {
  ($a:expr) => { delimited(multispace0,tag($a),multispace0) }
}

pub fn exprp<'a>(input: &'a str,prec: &HashMap<String,usize>) -> IResult<&'a str,Expr> {
  let (input,_) = multispace0(input)?;
  return alt((|x| apporlstp(x,prec),map(|x| constp(x,prec),|x| Expr::C(x))
             ,map(varp,|x| Expr::Var(x))))(input);
}

/*pub fn levelp<'a>(input: &'a str, prec: &HashMap<String,usize>) -> IResult<&'a str,Vec<Expr>> {
  return separated_list0(tag(";"),delimited(multispace0,|x| exprp(x,prec),multispace0))(input);
}*/

pub fn levelp<'a>(input: &'a str, prec: &HashMap<String,usize>) -> IResult<&'a str,Expr> {
  let (input,e) = exprp(input,prec)?;
  let (input,_) = stag!(";")(input)?;
  return Ok((input,e));
}

pub fn levelsp<'a>(input: &'a str, prec: &HashMap<String,usize>) -> IResult<&'a str,Vec<Expr>> {
  let (input,es) = many1(|x| levelp(x,prec))(input)?;
  return Ok((input,es));
}

/*pub enum Stmt {
  VarDef(String,Expr), SetPlat(Expr), Import(String), ModuleBegin(String,Expr,Box<Vec<Stmt>>)
, Open(String), Type(Expr,Vec<Expr>), E(String)
}*/

pub fn vardefp<'a>(input: &'a str, prec: &HashMap<String,usize>) -> IResult<&'a str,Stmt> {
  let (input,_) = delimited(multispace0,tag("var"),multispace1)(input)?;
  let (input,v) = varp(input)?;
  let (input,_) = delimited(multispace0,tag("=:"),multispace0)(input)?;
  let (input,e) = exprp(input,prec)?;
  return Ok((input,Stmt::VarDef(v,e)));
}

pub fn setplatp<'a>(input: &'a str, prec: &HashMap<String,usize>) -> IResult<&'a str,Stmt> {
  let (input,_) = delimited(multispace0,tag("platform"),multispace1)(input)?;
  let (input,e) = exprp(input,prec)?;
  return Ok((input,Stmt::SetPlat(e)));
}

pub fn importp<'a>(input: &'a str, prec: &HashMap<String,usize>) -> IResult<&'a str,Stmt> {
  let (input,_) = delimited(multispace0,tag("import"),multispace1)(input)?;
  let (input,s) = separated_list1(tag("/"),alphanumeric1)(input)?;
  let (input,prefix) = many0(delimited(stag!("!{"),|x| exprp(x,prec)
                                      ,stag!("}")))(input)?;
  return Ok((input,Stmt::Import(s.into_iter().map(|x| x.to_string()).collect(),prefix)));
}

pub fn modbeginp<'a>(input: &'a str, prec: &HashMap<String,usize>) -> IResult<&'a str,Stmt> {
  let (input,_) = delimited(multispace0,tag("module"),multispace1)(input)?;
  let (input,name) = alphanumeric1(input)?;
  let (input,prefix) = many0(delimited(stag!("!{"),|x| exprp(x,prec)
                                      ,stag!("}")))(input)?;
  let (input,es) = delimited(stag!("{"),|x| stmtsp(x,prec)
                            ,stag!("}"))(input)?;
  return Ok((input,Stmt::ModuleBegin(name.to_string(),prefix,Box::new(es))));
}

pub fn openp<'a>(input: &'a str, prec: &HashMap<String,usize>) -> IResult<&'a str,Stmt> {
  let (input,_) = delimited(multispace0,tag("open"),multispace1)(input)?;
  let (input,name) = alphanumeric1(input)?;
  return Ok((input,Stmt::Open(name.to_string())));
}

pub fn typep<'a>(input: &'a str, prec: &HashMap<String,usize>) -> IResult<&'a str,Stmt> {
  let (input,_) = delimited(multispace0,tag("type"),multispace0)(input)?;
  let (input,name) = delimited(stag!("{"),|x| exprp(x,prec),stag!("}"))(input)?;
  let (input,lst) = delimited(stag!("{")
                             ,|x| levelsp(x,prec)
                             ,stag!("}"))(input)?;
  return Ok((input,Stmt::Type(name,lst)));
}

pub fn stmtp<'a>(input: &'a str, prec: &HashMap<String,usize>) -> IResult<&'a str,Stmt> {
  let (input,q) = alt((|x| vardefp(x,prec),|x| setplatp(x,prec),|x| importp(x,prec)
                      ,|x| modbeginp(x,prec),|x| openp(x,prec)
                      ,|x| typep(x,prec),map(|x| exprp(x,prec),|x| Stmt::Ex(x))))(input)?;
  let (input,_) = delimited(multispace0,tag(";"),multispace0)(input)?;
  return Ok((input,q));
}

pub fn stmtsp<'a>(input: &'a str, prec: &HashMap<String,usize>) -> IResult<&'a str,Vec<Stmt>> {
  let (input,qs) = many1(|x| stmtp(x,prec))(input)?;
  return Ok((input,qs));
}
