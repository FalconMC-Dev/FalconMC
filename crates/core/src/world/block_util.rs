use crate::world::blocks::Blocks;

/// According to Fabric, these are the values that
/// are checked by the heightmap.
///
/// Corresponds to `material.blocksMovement()`.
pub fn blocks_movement(block: &Blocks) -> bool {
    !matches!(
        block,
        Blocks::BrownCarpet
            | Blocks::CrimsonButton(_)
            | Blocks::BrainCoralWallFan(_)
            | Blocks::OakSapling(_)
            | Blocks::HornCoralFan(_)
            | Blocks::YellowCarpet
            | Blocks::SmallDripleaf(_)
            | Blocks::Grass
            | Blocks::LilyPad
            | Blocks::ChorusFlower(_)
            | Blocks::BubbleColumn(_)
            | Blocks::CyanCarpet
            | Blocks::CreeperHead(_)
            | Blocks::PottedWhiteTulip
            | Blocks::NetherSprouts
            | Blocks::TallSeagrass(_)
            | Blocks::TwistingVinesPlant
            | Blocks::NetherPortal(_)
            | Blocks::PottedSpruceSapling
            | Blocks::VoidAir
            | Blocks::PottedBrownMushroom
            | Blocks::StructureVoid
            | Blocks::Potatoes(_)
            | Blocks::DarkOakSapling(_)
            | Blocks::PottedAzaleaBush
            | Blocks::PottedPoppy
            | Blocks::BubbleCoralWallFan(_)
            | Blocks::SpruceSapling(_)
            | Blocks::Snow(_)
            | Blocks::BigDripleaf(_)
            | Blocks::BrownMushroom
            | Blocks::SoulFire
            | Blocks::SpruceButton(_)
            | Blocks::PottedDeadBush
            | Blocks::Cobweb
            | Blocks::MagentaCarpet
            | Blocks::BlueOrchid
            | Blocks::PottedCrimsonFungus
            | Blocks::CaveAir
            | Blocks::FlowerPot
            | Blocks::KelpPlant
            | Blocks::HangingRoots(_)
            | Blocks::BirchSapling(_)
            | Blocks::AttachedPumpkinStem(_)
            | Blocks::SoulTorch
            | Blocks::JungleSapling(_)
            | Blocks::Kelp(_)
            | Blocks::ChorusPlant(_)
            | Blocks::Air
            | Blocks::Poppy
            | Blocks::AzureBluet
            | Blocks::CyanCandle(_)
            | Blocks::BrownCandle(_)
            | Blocks::WarpedFungus
            | Blocks::RedTulip
            | Blocks::PottedOakSapling
            | Blocks::WhiteCarpet
            | Blocks::Peony(_)
            | Blocks::GlowLichen(_)
            | Blocks::PottedBirchSapling
            | Blocks::StoneButton(_)
            | Blocks::Azalea
            | Blocks::PottedAllium
            | Blocks::SkeletonSkull(_)
            | Blocks::BubbleCoral(_)
            | Blocks::RedCandle(_)
            | Blocks::Lilac(_)
            | Blocks::BlackCandle(_)
            | Blocks::LightBlueCandle(_)
            | Blocks::BambooSapling
            | Blocks::DeadBush
            | Blocks::GreenCandle(_)
            | Blocks::MelonStem(_)
            | Blocks::Carrots(_)
            | Blocks::OrangeCarpet
            | Blocks::Cornflower
            | Blocks::LimeCarpet
            | Blocks::Rail(_)
            | Blocks::PottedOrangeTulip
            | Blocks::PottedCornflower
            | Blocks::BubbleCoralFan(_)
            | Blocks::Vine(_)
            | Blocks::PottedCrimsonRoots
            | Blocks::BrainCoralFan(_)
            | Blocks::OxeyeDaisy
            | Blocks::PottedWitherRose
            | Blocks::BlueCandle(_)
            | Blocks::PurpleCarpet
            | Blocks::Lava(_)
            | Blocks::WeepingVines(_)
            | Blocks::Beetroots(_)
            | Blocks::PottedBlueOrchid
            | Blocks::PottedLilyOfTheValley
            | Blocks::PinkCandle(_)
            | Blocks::PottedBamboo
            | Blocks::TallGrass(_)
            | Blocks::FireCoralFan(_)
            | Blocks::PottedDarkOakSapling
            | Blocks::TubeCoralWallFan(_)
            | Blocks::SugarCane(_)
            | Blocks::RedMushroom
            | Blocks::EndGateway
            | Blocks::PolishedBlackstoneButton(_)
            | Blocks::MossCarpet
            | Blocks::HornCoralWallFan(_)
            | Blocks::PottedRedTulip
            | Blocks::WeepingVinesPlant
            | Blocks::WitherSkeletonSkull(_)
            | Blocks::Sunflower(_)
            | Blocks::SporeBlossom
            | Blocks::DetectorRail(_)
            | Blocks::Allium
            | Blocks::PottedJungleSapling
            | Blocks::RedstoneWire(_)
            | Blocks::SweetBerryBush(_)
            | Blocks::FireCoralWallFan(_)
            | Blocks::WarpedButton(_)
            | Blocks::BirchButton(_)
            | Blocks::GrayCarpet
            | Blocks::RoseBush(_)
            | Blocks::LightGrayCandle(_)
            | Blocks::BlackCarpet
            | Blocks::Seagrass
            | Blocks::BlueCarpet
            | Blocks::MagentaCandle(_)
            | Blocks::PottedCactus
            | Blocks::BrainCoral(_)
            | Blocks::Cocoa(_)
            | Blocks::PottedOxeyeDaisy
            | Blocks::JungleButton(_)
            | Blocks::PottedAcaciaSapling
            | Blocks::Fire(_)
            | Blocks::Light(_)
            | Blocks::RedstoneTorch(_)
            | Blocks::PinkCarpet
            | Blocks::EndRod(_)
            | Blocks::PinkTulip
            | Blocks::FloweringAzalea
            | Blocks::LightBlueCarpet
            | Blocks::TripwireHook(_)
            | Blocks::Water(_)
            | Blocks::Candle(_)
            | Blocks::GreenCarpet
            | Blocks::WhiteCandle(_)
            | Blocks::OrangeCandle(_)
            | Blocks::LimeCandle(_)
            | Blocks::Dandelion
            | Blocks::EndPortal
            | Blocks::PoweredRail(_)
            | Blocks::OakButton(_)
            | Blocks::CaveVinesPlant(_)
            | Blocks::BigDripleafStem(_)
            | Blocks::ZombieHead(_)
            | Blocks::AttachedMelonStem(_)
            | Blocks::WarpedRoots
            | Blocks::NetherWart(_)
            | Blocks::WhiteTulip
            | Blocks::YellowCandle(_)
            | Blocks::Scaffolding(_)
            | Blocks::Torch
            | Blocks::Tripwire(_)
            | Blocks::AcaciaSapling(_)
            | Blocks::DragonHead(_)
            | Blocks::Lever(_)
            | Blocks::DarkOakButton(_)
            | Blocks::PottedRedMushroom
            | Blocks::CrimsonFungus
            | Blocks::Comparator(_)
            | Blocks::AcaciaButton(_)
            | Blocks::PottedWarpedRoots
            | Blocks::ActivatorRail(_)
            | Blocks::PurpleCandle(_)
            | Blocks::TubeCoral(_)
            | Blocks::WitherRose
            | Blocks::PottedDandelion
            | Blocks::PowderSnow
            | Blocks::PlayerHead(_)
            | Blocks::SeaPickle(_)
            | Blocks::LightGrayCarpet
            | Blocks::Ladder(_)
            | Blocks::RedCarpet
            | Blocks::PottedWarpedFungus
            | Blocks::PottedPinkTulip
            | Blocks::PottedFern
            | Blocks::LilyOfTheValley
            | Blocks::TwistingVines(_)
            | Blocks::OrangeTulip
            | Blocks::Wheat(_)
            | Blocks::PumpkinStem(_)
            | Blocks::CaveVines(_)
            | Blocks::TubeCoralFan(_)
            | Blocks::Repeater(_)
            | Blocks::HornCoral(_)
            | Blocks::LargeFern(_)
            | Blocks::PottedFloweringAzaleaBush
            | Blocks::GrayCandle(_)
            | Blocks::PottedAzureBluet
            | Blocks::CrimsonRoots
            | Blocks::Fern
            | Blocks::FireCoral(_)
    )
}
