use crate::parser::Parser;

mod keywords;
mod lexer;
mod parser;

pub fn parse(code: String) -> (Vec<Command>, Vec<String>) {
    let mut parser = Parser::new(&code);
    let mut commands = Vec::new();
    while let Some(command) = parser.command() {
        commands.push(command)
    }

    (commands, parser.finish())
}

#[derive(Debug, Clone, Copy)]
pub enum Command {
    MoveForward(f32),
    MoveBackward(f32),
    RotateRight(f32),
    RotateLeft(f32),
    FrontArmUp(f32),
    FrontArmDown(f32),
    BackArmUp(f32),
    BackArmDown(f32),
    PyDebug,
    Nop,
}

pub fn transpile(code: Vec<Command>) -> String {
    let mut script = String::new();
    for i in code {
        match i {
            Command::MoveForward(amount) => {
                script.push_str(format!("    await move({amount})").as_str())
            }
            Command::MoveBackward(amount) => {
                script.push_str(format!("    await move(-{amount})").as_str())
            }
            Command::RotateRight(amount) => {
                script.push_str(format!("    await rot({amount})").as_str())
            }
            Command::RotateLeft(amount) => {
                script.push_str(format!("    await rot(-{amount})").as_str())
            }
            Command::FrontArmUp(amount) => {
                script.push_str(format!("    await armF({amount})").as_str())
            }
            Command::FrontArmDown(amount) => {
                script.push_str(format!("    await armF(-{amount})").as_str())
            }
            Command::BackArmUp(amount) => {
                script.push_str(format!("    await armB({amount})").as_str())
            }
            Command::BackArmDown(amount) => {
                script.push_str(format!("    await armB(-{amount})").as_str())
            }
            Command::PyDebug => script.push_str("    await debug()"),
            Command::Nop => {}
        }
        script.push('\n');
    }
    script.push_str("    pass");

    PYTHON_SHELL.replace("<SCRIPT>", &script[..])
}

const PYTHON_SHELL: &str = r#"from hub import light_matrix
from hub import port
from hub import sound
import motor_pair
import motor
import runloop
import math

SPEED = 1110
ARMF = port.E
ARMB = port.F
LEFT = port.C
RIGHT = port.D

async def main():
    motor_pair.pair(motor_pair.PAIR_1, LEFT, RIGHT)
    await script()

async def armF(deg: float):
    motor.run_for_degrees(ARMF, int(deg), SPEED)

async def armB(deg: float):
    motor.run_for_degrees(ARMB, int(deg), SPEED)

CIRC = 17.5

async def move(cm: float):
    await motor_pair.move_for_degrees(motor_pair.PAIR_1, int(cm / CIRC * 360), 0, velocity=SPEED)

async def rot(deg: float):
    motor.run_for_degrees(LEFT, -int(deg * 2), SPEED)
    await motor.run_for_degrees(RIGHT, -int(deg * 2), SPEED)

async def debug():
    await light_matrix.write("This is a debug message.")

async def script():
<SCRIPT>
runloop.run(main())"#;
