use agb::{
    display::{
        Priority,
        object::SpriteLoader,
        tiled::{RegularBackgroundSize, TiledMap},
    },
    include_background_gfx,
};
use bevy::prelude::*;
use bevy_mod_gba::{Sprite, SpriteHandles, Video};

use crate::Sprites;

include_background_gfx!(generated_background, "000000", DATA => "assets/test_logo_basic.png");

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Mel0nGbaSetupSet;

pub struct Mel0nGbaPlugin;

impl Plugin for Mel0nGbaPlugin {
    fn build(&self, app: &mut App) {
        app.init_non_send_resource::<Option<Sprites>>().add_systems(
            Startup,
            (setup_video, load_sprites).chain().in_set(Mel0nGbaSetupSet),
        );
    }
}

fn setup_video(mut video: ResMut<Video>) {
    let (background, mut vram) = video.tiled0();

    let mut map = background.background(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        generated_background::DATA.tiles.format(),
    );
    vram.set_background_palettes(generated_background::PALETTES);

    map.fill_with(&mut vram, &generated_background::DATA);
    map.commit(&mut vram);
    map.set_visible(true);
}

fn load_sprites(
    mut loader: NonSendMut<SpriteLoader>,
    mut handles: NonSendMut<SpriteHandles>,
    mut sprites: NonSendMut<Option<Sprites>>,
) -> Result<()> {
    static GRAPHICS: &agb::display::object::Graphics =
        agb::include_aseprite!("./assets/fruits.aseprite");

    let hero: &agb::display::object::Sprite = GRAPHICS.sprites().get(3).ok_or("Damn")?;

    let vram = loader.get_vram_sprite(hero);

    let handle = handles.add(vram);

    let player = Sprite::new(handle);

    *sprites = Some(Sprites { player });

    Ok(())
}
