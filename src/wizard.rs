//! With the wizard a configuration can be created interactively
//! There are two main cases:
//! A) A new configuration should be generated
//! B) A existing configuration should be updated
use ::*;
use std::str;
use std::fmt::{Display, Debug};
use console::Term;
use colored::*;


pub trait Dialog {
    fn ask() -> Self;
    fn compare(&self, other: Option<&Self>) -> Self;
}

/// Compare two items of same type to each other and display them besides each\
/// other. Display a selector
pub fn differ<T>(this: T, opt_that: Option<T>) -> T 
    where T: 'static + PartialEq + Display + Clone + Debug + std::str::FromStr,
    <T as std::str::FromStr>::Err: Display + Debug{
    match opt_that{
        Some(that) => {
            if this != that{
                compareprint(&this, &that, width(), "!=");
                let choice = Select::new()
                                .item(format!("{}", this).as_str())
                                .item(format!("{}", that).as_str())
                                .item("Manual override")
                                .default(0)
                                .interact()
                                .expect("Couldn't display dialog.");
                match choice{
                    2 => {
                        let x: T = Input::<T>::new().default(this).interact().expect("Couldn't display dialog.");
                        x
                    }
                    1 => that,
                    _ => this
                }
            }else{
                this
            } 
        },
        None      => {
            println!("{}{}", " Existing value -> ".black().on_yellow(), format!(" {} ", &this).black().on_green());
            let choice = Select::new()
                            .item("Keep")
                            .item("Manual override")
                            .default(0)
                            .interact()
                            .expect("Couldn't display dialog.");
            match choice{
                1 => {
                    let x: T = Input::<T>::new().default(this).interact().expect("Couldn't display dialog.");
                    x
                }
                _ => this
            }
        }
    }
}

/// Print two elements for comparison
fn compareprint<T, S>(left: T, right: T, w: usize, sign: S) 
where T: Display, S: Into<String>{
    let sign = sign.into();
    let left = format!("{}", left);
    let right = format!("{}", right);
    let halfwidth = w/2-sign.chars().count()/2;
    let left = wrap(left, halfwidth-4);
    let right = wrap(right, halfwidth-4);
    let mut leftdots = "";
    let mut rightdots = "";
    if left.len() >1 {leftdots = "..."};
    if right.len() >1 {rightdots = "..."};
    println!("{left: <0}{leftspacer}{sign}{rightspacer}{right}",
        left=format!("{}{}", left[0], leftdots),
        leftspacer=" ".repeat(halfwidth-left[0].chars().count()-leftdots.chars().count()),
        sign=sign,
        rightspacer=" ".repeat(halfwidth-right[0].chars().count()-rightdots.chars().count()),
        right=format!("{}{}", right[0], rightdots));
 }


/// Wrap a string at the given index stupidly, return lines as a Vec<String>
fn wrap<S>(s: S, w: usize) -> Vec<String> where S: Into<String>{
    let s = s.into();
    let s = s.as_str();
    s.as_bytes()
     .chunks(w)
     .map(str::from_utf8)
     .filter(|res|res.is_ok())
     .map(|st|st.unwrap().to_string())
     .collect()
}


/// Return the width of the terminal
fn width() -> usize{
    let term = Term::stdout();
    term.size().1 as usize
}




// Print a errorlabel
#[allow(dead_code)]
pub fn errorprint<S>(s: S) where S: Into<String>{
    let s = s.into();
    let label = " Error ".on_red().bold();
    eprintln!("    {} {}", label, s);
}

// Print a errorlabel
#[allow(dead_code)]
pub fn okprint<S>(s: S) where S: Into<String>{
    let s = s.into();
    let label = "   OK  ".on_green().bold();
    println!("    {} {}", label, s);
}


/// Print a section label like ---------------- foo ----------------
pub fn print_sectionlabel<S>(message: S) where S: Into<String>{
    let message = message.into();
    let screen_width = width();
    println!("{}", "-".repeat(screen_width));
    println!("{:^width$}", message, width=screen_width);
    println!("{}", "-".repeat(screen_width));
}

/// Print a a block
pub fn print_block<S>(message: S) where S: Into<String>{
    let message = message.into();
    println!("{}", message.black().on_bright_white());
}




