use yabaictl::yabai::{cli::execute_yabai_cmd, command::FocusSpaceByIndex, transport::SpaceIndex};

fn main() {
    execute_yabai_cmd(&FocusSpaceByIndex {
        index: SpaceIndex(5),
    })
    .expect("could not execute yabai cmd");
    // let output = Command::new("yabai")
    //     .args(["-m", "query", "--spaces"])
    //     .output()
    //     .expect("could not run yabai");
    // let output = String::from_utf8(output.stdout).expect("output is not valid utf-8");
    // let spaces = serde_json::from_str::<Vec<Space>>(&output).expect("could not parse result");
    //
    // for space in spaces {
    //     println!("Space {} has {} windows", space.index, space.windows.len());
    // }
}
