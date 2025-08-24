mod bravery;
mod bind;
mod recent;

pub fn all_commands() -> Vec<crate::Command> {
    vec![bravery::bravery(), bind::bind(), recent::recent()]
}
