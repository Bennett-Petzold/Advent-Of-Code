use std::{mem::ManuallyDrop, rc::Rc};

use advent_rust_lib::read::input;
use enum_primitive::{enum_from_primitive, FromPrimitive};
use z3::{ast::BV, SatResult};

use z3::ast::{Ast, Int as z3Int};

#[macro_use]
extern crate enum_primitive;

fn main() {
    let computer = Computer::setup(input()).unwrap();
    part_1(computer.clone());
    part_2(computer.clone());
}

fn part_1(mut computer: Computer) {
    computer.exec_full();

    let output = computer.take_output();
    for line in &output[..output.len() - 1] {
        print!("{line},")
    }
    if let Some(last) = output.last() {
        println!("{last}")
    }
}

fn part_2(computer: Computer) {
    let mut solves = vec![ComputerSolve::from_known(computer)];

    let mut idx = 0;
    loop {
        solves = solves
            .into_iter()
            .filter(|solve| !solve.is_finished())
            .flat_map(|solve| solve.exec_one().into_iter())
            .flatten()
            //.filter(|solve| solve.is_valid())
            .collect();

        if let Some(resolved) = solves
            .iter()
            .filter(|solve| solve.is_finished() && solve.produced_all_outputs())
            .inspect(|_| println!("CONDITION MET"))
            .filter_map(|solve| solve.solution())
            .next()
        {
            println!("{resolved}");
            break;
        } else {
            idx += 1;
            if idx % 10 == 0 {
                println!("Idx: {idx}");
                println!("Solves len: {}", solves.len());
            }
            debug_assert!(!solves.is_empty());
        }
    }
}

#[derive(Debug, Clone)]
pub struct Computer {
    // A, B, C
    registers: [u64; 3],
    pub program: Box<[u8]>,
    pc: usize,
    output_buffer: Vec<u8>,
}

impl Computer {
    pub fn setup<S, I>(iter: I) -> Option<Self>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut iter = iter.into_iter();

        let register_vec: Vec<_> = iter
            .by_ref()
            .map(|s| {
                let s = s.as_ref();
                let start_slice = s.find(':')?;
                str::parse::<u64>(&s[start_slice + 2..]).ok()
            })
            .take(3)
            .collect::<Option<_>>()?;
        if register_vec.len() < 3 {
            return None;
        }

        iter.next();

        let s = iter.next()?;
        let s = s.as_ref();
        let start_slice = s.find(':')?;
        let program = s[start_slice + 2..]
            .split(',')
            .map(|num| str::parse::<u8>(num).ok())
            .collect::<Option<_>>()?;

        Some(Self {
            registers: [register_vec[0], register_vec[1], register_vec[2]],
            program,
            pc: 0,
            output_buffer: Vec::new(),
        })
    }

    pub fn exec_one(&mut self) {
        let arg = self.program[self.pc + 1];
        match Instruction::from_u8(self.program[self.pc]).expect("All opcodes [0, 7] are valid") {
            Instruction::Adv => {
                let arg = if arg < 4 {
                    arg as u64
                } else {
                    self.registers[(arg - 4) as usize]
                };

                let pow_2 = 1 << arg;
                self.registers[0] /= pow_2;
                self.pc += 2;
            }
            Instruction::Bxl => {
                self.registers[1] ^= arg as u64;
                self.pc += 2;
            }
            Instruction::Bst => {
                let arg = if arg < 4 {
                    arg as u64
                } else {
                    self.registers[(arg - 4) as usize]
                };

                self.registers[1] = arg % 8;
                self.pc += 2;
            }
            Instruction::Jnz => {
                if self.registers[0] != 0 {
                    self.pc = arg as usize;
                } else {
                    self.pc += 2;
                }
            }
            Instruction::Bxc => {
                self.registers[1] ^= self.registers[2];
                self.pc += 2;
            }
            Instruction::Out => {
                let arg = if arg < 4 {
                    arg as u64
                } else {
                    self.registers[(arg - 4) as usize]
                };

                self.output_buffer.push((arg % 8) as u8);
                self.pc += 2;
            }
            Instruction::Bdv => {
                let arg = if arg < 4 {
                    arg as u64
                } else {
                    self.registers[(arg - 4) as usize]
                };

                let pow_2 = 1 << arg;
                self.registers[1] = self.registers[0] / pow_2;
                self.pc += 2;
            }
            Instruction::Cdv => {
                let arg = if arg < 4 {
                    arg as u64
                } else {
                    self.registers[(arg - 4) as usize]
                };

                let pow_2 = 1 << arg;
                self.registers[2] = self.registers[0] / pow_2;
                self.pc += 2;
            }
        }
    }

    pub fn exec_full(&mut self) {
        while !self.is_finished() {
            self.exec_one();
        }
    }

    pub fn is_finished(&self) -> bool {
        (self.pc + 1) >= self.program.len()
    }

    pub fn take_output(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.output_buffer)
    }

    pub fn pop_output(&mut self) -> Option<u8> {
        self.output_buffer.pop()
    }
}

enum_from_primitive! {
    pub enum Instruction {
        Adv = 0,
        Bxl = 1,
        Bst = 2,
        Jnz = 3,
        Bxc = 4,
        Out = 5,
        Bdv = 6,
        Cdv = 7,
    }
}

#[derive(Debug)]
pub struct ComputerSolve {
    // A, B, C
    cfg: ManuallyDrop<Rc<z3::Config>>,
    ctx: &'static Rc<z3::Context>,
    consts: ManuallyDrop<[z3Int<'static>; 8]>,
    eight: ManuallyDrop<z3Int<'static>>,
    reg_a: ManuallyDrop<z3Int<'static>>,
    registers: ManuallyDrop<[z3Int<'static>; 3]>,
    pub program: Rc<[u8]>,
    pc: usize,
    solver: ManuallyDrop<z3::Solver<'static>>,
    next_program_output: usize,
}

impl Clone for ComputerSolve {
    fn clone(&self) -> Self {
        Self {
            cfg: self.cfg.clone(),
            ctx: Box::leak(Box::new(self.ctx.clone())),
            consts: self.consts.clone(),
            eight: self.eight.clone(),
            reg_a: self.reg_a.clone(),
            registers: self.registers.clone(),
            program: self.program.clone(),
            pc: self.pc,
            solver: self.solver.clone(),
            next_program_output: self.next_program_output,
        }
    }
}

impl ComputerSolve {
    pub fn from_known(computer: Computer) -> Self {
        let cfg = Rc::new(z3::Config::new());
        // Need a static lifetime for borrow
        let ctx = Box::leak(Box::new(Rc::new(z3::Context::new(&cfg))));

        let reg_a = z3Int::new_const(ctx, "A");
        let reg_b = z3Int::from_u64(ctx, computer.registers[1]);
        let reg_c = z3Int::from_u64(ctx, computer.registers[2]);

        let solver = z3::Solver::new(ctx);

        Self {
            cfg: ManuallyDrop::new(cfg),
            ctx,
            reg_a: ManuallyDrop::new(reg_a.clone()),
            consts: ManuallyDrop::new(std::array::from_fn(|idx| z3Int::from_u64(ctx, idx as u64))),
            eight: ManuallyDrop::new(z3Int::from_u64(ctx, 8)),
            registers: ManuallyDrop::new([reg_a, reg_b, reg_c]),
            program: computer.program.into(),
            pc: 0,
            solver: ManuallyDrop::new(solver),
            next_program_output: 0,
        }
    }

    pub fn exec_one(mut self) -> [Option<Self>; 2] {
        const BV_WIDTH: u32 = 64;

        let arg = self.program[self.pc + 1];
        match Instruction::from_u8(self.program[self.pc]).expect("All opcodes [0, 7] are valid") {
            Instruction::Adv => {
                let arg = if arg < 4 {
                    &self.consts[arg as usize]
                } else {
                    &self.registers[(arg - 4) as usize]
                };

                let two = &self.consts[2];
                self.registers[0] = self.registers[0].div(&z3Int::from_real(&two.power(arg)));
                self.pc += 2;
            }
            Instruction::Bxl => {
                let b_bv = BV::from_int(&self.registers[1], BV_WIDTH);
                let arg_bv = BV::from_int(&self.consts[arg as usize], BV_WIDTH);
                self.registers[1] = z3Int::from_bv(&b_bv.bvxor(&arg_bv), false);
                self.pc += 2;
            }
            Instruction::Bst => {
                let arg = if arg < 4 {
                    &self.consts[arg as usize]
                } else {
                    &self.registers[(arg - 4) as usize]
                };

                self.registers[1] = arg.modulo(&self.eight);
                self.pc += 2;
            }
            Instruction::Jnz => {
                // Second copy that did the jump
                let mut jump_version = self.clone();
                jump_version
                    .solver
                    .assert(&jump_version.registers[0]._eq(&jump_version.consts[0]).not());
                jump_version.pc = arg as usize;

                // This copy did not do the jump
                self.solver.assert(&self.registers[0]._eq(&self.consts[0]));
                self.pc += 2;

                return [Some(self), Some(jump_version)];
            }
            Instruction::Bxc => {
                let b_bv = BV::from_int(&self.registers[1], BV_WIDTH);
                let c_bv = BV::from_int(&self.registers[2], BV_WIDTH);
                self.registers[1] = z3Int::from_bv(&b_bv.bvxor(&c_bv), false);
                self.pc += 2;
            }
            Instruction::Out => {
                let arg = if arg < 4 {
                    &self.consts[arg as usize]
                } else {
                    &self.registers[(arg - 4) as usize]
                };

                if let Some(next_out) = self.program.get(self.next_program_output) {
                    self.solver.assert(
                        &arg.modulo(&self.eight)
                            ._eq(&z3Int::from_u64(self.ctx, *next_out as u64)),
                    );

                    self.next_program_output += 1;
                    self.pc += 2;
                } else {
                    return [None, None];
                }
            }
            Instruction::Bdv => {
                let arg = if arg < 4 {
                    &self.consts[arg as usize]
                } else {
                    &self.registers[(arg - 4) as usize]
                };

                let two = &self.consts[2];
                self.registers[1] = self.registers[0].div(&z3Int::from_real(&two.power(arg)));
                self.pc += 2;
            }
            Instruction::Cdv => {
                let arg = if arg < 4 {
                    &self.consts[arg as usize]
                } else {
                    &self.registers[(arg - 4) as usize]
                };

                let two = &self.consts[2];
                self.registers[2] = self.registers[0].div(&z3Int::from_real(&two.power(arg)));
                self.pc += 2;
            }
        }

        [Some(self), None]
    }

    pub fn is_valid(&self) -> bool {
        self.solver.check() != SatResult::Unsat
    }

    pub fn is_finished(&self) -> bool {
        (self.pc + 1) >= self.program.len()
    }

    pub fn produced_all_outputs(&self) -> bool {
        self.next_program_output == self.program.len()
    }

    pub fn solution(&self) -> Option<u64> {
        if self.solver.check() == SatResult::Sat {
            self.solver
                .get_model()
                .and_then(|model| model.get_const_interp(&*self.reg_a))
                .and_then(|val| val.as_u64())
        } else {
            None
        }
    }
}

impl Drop for ComputerSolve {
    fn drop(&mut self) {
        // Drop all fields reliant on ctx, and then drop ctx
        unsafe {
            ManuallyDrop::drop(&mut self.cfg);
            ManuallyDrop::drop(&mut self.consts);
            ManuallyDrop::drop(&mut self.eight);
            ManuallyDrop::drop(&mut self.reg_a);
            ManuallyDrop::drop(&mut self.registers);
            ManuallyDrop::drop(&mut self.solver);

            let ctx_ptr = Box::from_raw(std::ptr::from_ref(self.ctx) as *mut Rc<z3::Context>);
            drop(ctx_ptr);
        }
    }
}
