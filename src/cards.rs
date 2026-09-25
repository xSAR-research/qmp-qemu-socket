use thiserror::Error;

use crate::geometry::{
    GeometryError, PixelPoint, PixelRect, QmpPoint, QmpRect, pixel_point_to_qmp, pixel_rect_to_qmp,
};

pub const TABLEAU_CARD_COUNT: usize = 28;
pub const TABLEAU_ROW_LENGTHS: [u8; 4] = [3, 6, 9, 10];
pub const TABLEAU_ROW_STARTS: [usize; 4] = [0, 3, 9, 18];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardRank {
    #[default]
    Unknown,
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

impl CardRank {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Unknown => " ",
            Self::Ace => "A",
            Self::Two => "2",
            Self::Three => "3",
            Self::Four => "4",
            Self::Five => "5",
            Self::Six => "6",
            Self::Seven => "7",
            Self::Eight => "8",
            Self::Nine => "9",
            Self::Ten => "10",
            Self::Jack => "J",
            Self::Queen => "Q",
            Self::King => "K",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardVisibility {
    #[default]
    Unobserved,
    Empty,
    FaceDown,
    FaceUp,
    Removed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardRegionPixels {
    pub card_bounds: PixelRect,
    pub halo_bounds: PixelRect,
    pub rank_bounds: PixelRect,
    pub click_point: PixelPoint,
}

impl CardRegionPixels {
    pub const fn new(
        card_bounds: PixelRect,
        halo_bounds: PixelRect,
        rank_bounds: PixelRect,
        click_point: PixelPoint,
    ) -> Self {
        Self {
            card_bounds,
            halo_bounds,
            rank_bounds,
            click_point,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardRegionQmp {
    pub card_bounds: QmpRect,
    pub halo_bounds: QmpRect,
    pub rank_bounds: QmpRect,
    pub click_point: QmpPoint,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardRegions {
    pixels: CardRegionPixels,
    qmp: CardRegionQmp,
}

impl CardRegions {
    pub fn from_pixels(
        pixels: CardRegionPixels,
        frame_width: u32,
        frame_height: u32,
    ) -> Result<Self, CardRegionError> {
        let card_bounds = pixel_rect_to_qmp(pixels.card_bounds, frame_width, frame_height)?;
        let halo_bounds = pixel_rect_to_qmp(pixels.halo_bounds, frame_width, frame_height)?;
        let rank_bounds = pixel_rect_to_qmp(pixels.rank_bounds, frame_width, frame_height)?;
        let click_point = pixel_point_to_qmp(pixels.click_point, frame_width, frame_height)?;

        if !rect_contains(pixels.halo_bounds, pixels.card_bounds) {
            return Err(CardRegionError::CardOutsideHalo);
        }
        if !rect_contains(pixels.card_bounds, pixels.rank_bounds) {
            return Err(CardRegionError::RankOutsideCard);
        }
        if !pixels.card_bounds.contains(pixels.click_point) {
            return Err(CardRegionError::ClickOutsideCard);
        }

        Ok(Self {
            qmp: CardRegionQmp {
                card_bounds,
                halo_bounds,
                rank_bounds,
                click_point,
            },
            pixels,
        })
    }

    pub const fn pixels(self) -> CardRegionPixels {
        self.pixels
    }

    pub const fn qmp(self) -> CardRegionQmp {
        self.qmp
    }
}

fn rect_contains(outer: PixelRect, inner: PixelRect) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.right() <= outer.right()
        && inner.bottom() <= outer.bottom()
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CardRegionError {
    #[error(transparent)]
    Geometry(#[from] GeometryError),
    #[error("card bounds must lie inside halo bounds")]
    CardOutsideHalo,
    #[error("rank bounds must lie inside card bounds")]
    RankOutsideCard,
    #[error("click point must lie inside card bounds")]
    ClickOutsideCard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableauPosition {
    row: u8,
    column: u8,
}

impl TableauPosition {
    pub fn new(row: u8, column: u8) -> Result<Self, TableauPositionError> {
        let row_index = row
            .checked_sub(1)
            .filter(|index| (*index as usize) < TABLEAU_ROW_LENGTHS.len())
            .ok_or(TableauPositionError { row, column })?;
        let row_length = TABLEAU_ROW_LENGTHS[row_index as usize];
        if column == 0 || column > row_length {
            return Err(TableauPositionError { row, column });
        }

        Ok(Self { row, column })
    }

    pub const fn row(self) -> u8 {
        self.row
    }

    pub const fn column(self) -> u8 {
        self.column
    }

    pub const fn index(self) -> usize {
        TABLEAU_ROW_STARTS[self.row as usize - 1] + self.column as usize - 1
    }

    pub fn from_index(index: usize) -> Option<Self> {
        TABLEAU_ROW_STARTS
            .iter()
            .zip(TABLEAU_ROW_LENGTHS)
            .enumerate()
            .find_map(|(row_index, (start, length))| {
                let end = *start + length as usize;
                (index >= *start && index < end).then_some(Self {
                    row: row_index as u8 + 1,
                    column: (index - *start) as u8 + 1,
                })
            })
    }
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq)]
#[error("invalid tableau position row {row}, column {column}")]
pub struct TableauPositionError {
    pub row: u8,
    pub column: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardLocation {
    Tableau(TableauPosition),
    Stock,
    Waste,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardSlot {
    location: CardLocation,
    regions: CardRegions,
}

impl CardSlot {
    pub fn from_pixels(
        location: CardLocation,
        pixels: CardRegionPixels,
        frame_width: u32,
        frame_height: u32,
    ) -> Result<Self, CardRegionError> {
        Ok(Self {
            location,
            regions: CardRegions::from_pixels(pixels, frame_width, frame_height)?,
        })
    }

    pub const fn location(self) -> CardLocation {
        self.location
    }

    pub const fn regions(self) -> CardRegions {
        self.regions
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CardObservation {
    slot: CardSlot,
    pub visibility: CardVisibility,
    pub rank: CardRank,
    pub has_halo: bool,
}

impl CardObservation {
    pub const fn new(
        slot: CardSlot,
        visibility: CardVisibility,
        rank: CardRank,
        has_halo: bool,
    ) -> Self {
        Self {
            slot,
            visibility,
            rank,
            has_halo,
        }
    }

    pub const fn unobserved(slot: CardSlot) -> Self {
        Self::new(slot, CardVisibility::Unobserved, CardRank::Unknown, false)
    }

    pub const fn slot(self) -> CardSlot {
        self.slot
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TableauState {
    cards: [CardObservation; TABLEAU_CARD_COUNT],
}

impl TableauState {
    pub fn from_calibration(
        calibration: &[CardRegionPixels; TABLEAU_CARD_COUNT],
        frame_width: u32,
        frame_height: u32,
    ) -> Result<Self, CardModelError> {
        let mut cards = Vec::with_capacity(TABLEAU_CARD_COUNT);
        for (index, pixels) in calibration.iter().copied().enumerate() {
            let position = TableauPosition::from_index(index)
                .ok_or(CardModelError::InvalidCalibrationCount)?;
            let slot = CardSlot::from_pixels(
                CardLocation::Tableau(position),
                pixels,
                frame_width,
                frame_height,
            )?;
            cards.push(CardObservation::unobserved(slot));
        }

        let cards = cards
            .try_into()
            .map_err(|_| CardModelError::InvalidCalibrationCount)?;
        Ok(Self { cards })
    }

    pub fn card(&self, position: TableauPosition) -> &CardObservation {
        &self.cards[position.index()]
    }

    pub fn record(&mut self, card: CardObservation) -> Result<(), CardModelError> {
        let CardLocation::Tableau(position) = card.slot().location() else {
            return Err(CardModelError::NonTableauObservation);
        };
        let target = &mut self.cards[position.index()];
        if card.slot() != target.slot() {
            return Err(CardModelError::TableauSlotMismatch);
        }
        *target = card;
        Ok(())
    }

    pub const fn cards(&self) -> &[CardObservation; TABLEAU_CARD_COUNT] {
        &self.cards
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DrawRecord {
    pub sequence: usize,
    pub card: CardObservation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawPileState {
    stock: CardObservation,
    waste: CardObservation,
    history: Vec<DrawRecord>,
}

impl DrawPileState {
    pub fn from_calibration(
        stock: CardRegionPixels,
        waste: CardRegionPixels,
        frame_width: u32,
        frame_height: u32,
    ) -> Result<Self, CardModelError> {
        Ok(Self {
            stock: CardObservation::unobserved(CardSlot::from_pixels(
                CardLocation::Stock,
                stock,
                frame_width,
                frame_height,
            )?),
            waste: CardObservation::unobserved(CardSlot::from_pixels(
                CardLocation::Waste,
                waste,
                frame_width,
                frame_height,
            )?),
            history: Vec::new(),
        })
    }

    pub fn record_waste(&mut self, card: CardObservation) -> Result<usize, CardModelError> {
        if card.slot() != self.waste.slot() {
            return Err(CardModelError::WasteSlotMismatch);
        }

        let sequence = self.history.len();
        self.waste = card;
        self.history.push(DrawRecord { sequence, card });
        Ok(sequence)
    }

    pub fn record_stock(&mut self, card: CardObservation) -> Result<(), CardModelError> {
        if card.slot() != self.stock.slot() {
            return Err(CardModelError::StockSlotMismatch);
        }
        self.stock = card;
        Ok(())
    }

    pub const fn stock(&self) -> &CardObservation {
        &self.stock
    }

    pub const fn waste(&self) -> &CardObservation {
        &self.waste
    }

    pub fn history(&self) -> &[DrawRecord] {
        &self.history
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CardModelError {
    #[error(transparent)]
    Region(#[from] CardRegionError),
    #[error("tableau calibration does not contain exactly 28 slots")]
    InvalidCalibrationCount,
    #[error("draw history observation does not use the calibrated waste slot")]
    WasteSlotMismatch,
    #[error("stock observation does not use the calibrated stock slot")]
    StockSlotMismatch,
    #[error("tableau update was given a non-tableau observation")]
    NonTableauObservation,
    #[error("tableau observation does not use its calibrated slot")]
    TableauSlotMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_region(x: u32, y: u32) -> CardRegionPixels {
        let card_bounds = PixelRect::new(x, y, 100, 150);
        CardRegionPixels::new(
            card_bounds,
            PixelRect::new(x - 5, y - 5, 110, 160),
            PixelRect::new(x + 3, y + 7, 25, 22),
            card_bounds.centre(),
        )
    }

    #[test]
    fn rank_labels_include_unknown_and_ten_without_char_sentinels() {
        assert_eq!(CardRank::Unknown.label(), " ");
        assert_eq!(CardRank::Ace.label(), "A");
        assert_eq!(CardRank::Ten.label(), "10");
        assert_eq!(CardRank::King.label(), "K");
    }

    #[test]
    fn tableau_positions_are_one_based_and_ragged() {
        assert!(TableauPosition::new(0, 1).is_err());
        assert!(TableauPosition::new(1, 4).is_err());
        assert!(TableauPosition::new(4, 10).is_ok());
        assert!(TableauPosition::new(4, 11).is_err());

        let last = TableauPosition::new(4, 10).unwrap();
        assert_eq!(last.index(), 27);
        assert_eq!(TableauPosition::from_index(27), Some(last));
        assert_eq!(TableauPosition::from_index(28), None);
    }

    #[test]
    fn tableau_initialises_exactly_twenty_eight_unknown_cards() {
        let calibration = std::array::from_fn(|index| {
            let x = 20 + (index % 10) as u32 * 110;
            let y = 20 + (index / 10) as u32 * 170;
            test_region(x, y)
        });
        let tableau = TableauState::from_calibration(&calibration, 1_920, 1_080).unwrap();

        assert_eq!(tableau.cards().len(), TABLEAU_CARD_COUNT);
        assert!(tableau.cards().iter().all(|card| {
            card.visibility == CardVisibility::Unobserved
                && card.rank == CardRank::Unknown
                && !card.has_halo
        }));
    }

    #[test]
    fn draw_history_preserves_repeated_ranks_in_action_order() {
        let stock = test_region(100, 600);
        let waste = test_region(300, 600);
        let mut draw = DrawPileState::from_calibration(stock, waste, 1_920, 1_080).unwrap();
        let mut first = *draw.waste();
        first.visibility = CardVisibility::FaceUp;
        first.rank = CardRank::Five;
        let second = first;

        assert_eq!(draw.record_waste(first), Ok(0));
        assert_eq!(draw.record_waste(second), Ok(1));
        assert_eq!(draw.history().len(), 2);
        assert_eq!(draw.history()[0].card.rank, CardRank::Five);
        assert_eq!(draw.history()[1].card.rank, CardRank::Five);
    }

    #[test]
    fn card_geometry_rejects_non_nested_regions_and_clicks() {
        let mut pixels = test_region(100, 100);
        pixels.rank_bounds = PixelRect::new(50, 50, 25, 22);
        assert_eq!(
            CardRegions::from_pixels(pixels, 1_920, 1_080),
            Err(CardRegionError::RankOutsideCard),
        );

        let mut pixels = test_region(100, 100);
        pixels.halo_bounds = PixelRect::new(110, 110, 90, 100);
        assert_eq!(
            CardRegions::from_pixels(pixels, 1_920, 1_080),
            Err(CardRegionError::CardOutsideHalo),
        );

        let mut pixels = test_region(100, 100);
        pixels.click_point = PixelPoint::new(99, 100);
        assert_eq!(
            CardRegions::from_pixels(pixels, 1_920, 1_080),
            Err(CardRegionError::ClickOutsideCard),
        );
    }

    #[test]
    fn draw_history_rejects_an_observation_from_another_slot() {
        let stock = test_region(100, 600);
        let waste = test_region(300, 600);
        let mut draw = DrawPileState::from_calibration(stock, waste, 1_920, 1_080).unwrap();

        let stock_observation = *draw.stock();
        assert_eq!(
            draw.record_waste(stock_observation),
            Err(CardModelError::WasteSlotMismatch),
        );
        assert!(draw.history().is_empty());
    }

    #[test]
    fn tableau_updates_preserve_the_calibrated_slot() {
        let calibration = std::array::from_fn(|index| {
            let x = 20 + (index % 10) as u32 * 110;
            let y = 20 + (index / 10) as u32 * 170;
            test_region(x, y)
        });
        let mut tableau = TableauState::from_calibration(&calibration, 1_920, 1_080).unwrap();
        let position = TableauPosition::new(4, 10).unwrap();
        let mut observed = *tableau.card(position);
        observed.visibility = CardVisibility::FaceUp;
        observed.rank = CardRank::Ten;
        observed.has_halo = true;

        assert_eq!(tableau.record(observed), Ok(()));
        assert_eq!(tableau.card(position).rank, CardRank::Ten);
        assert!(tableau.card(position).has_halo);

        let wrong_slot = CardSlot::from_pixels(
            CardLocation::Tableau(position),
            test_region(1_200, 700),
            1_920,
            1_080,
        )
        .unwrap();
        assert_eq!(
            tableau.record(CardObservation::unobserved(wrong_slot)),
            Err(CardModelError::TableauSlotMismatch),
        );
    }
}
