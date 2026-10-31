//! Source identity disappearance, independent of replacement identity and RGB motion.

use super::*;
use crate::pyramid::PyramidTargetKind;


/// One verified original identity at its exact calibrated location.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SourceCard {
    /// Original tableau or stock location, never a replacement location.
    pub(crate) location: PyramidTargetKind,
    /// Identity that must cease being identified after the single action.
    pub(crate) card: Card,
}


/// Two successive fresh observations of all original sources disappearing.
#[derive(Debug)]
pub(crate) struct SourceTransition {
    /// At most two source cards belonging to one logical action.
    sources: Vec<SourceCard>,
    /// Last actual capture serial, including refused observations.
    capture: usize,
    /// Consecutive supported observations without any original identity.
    confirmations: usize,
}


impl SourceTransition {
    /// Verify the expected sources on the existing pre-input frame.
    pub(crate) fn begin(
        vision: &ReconVision,
        frame: &CapturedFrame,
        sources: Vec<SourceCard>,
        capture: usize,
    ) -> Result<Self, String> {
        let (transition, reads) = Self::prepare(vision, frame, sources, capture)?;
        transition.ok_or_else(|| {
            format!(
                "expected source is not identified; no input authorised: {}",
                reads.join("; ")
            )
        })
    }


    /// Check every expected source on one supported frame; nonmatches stay pending.
    /// Structural/layout/scene errors remain errors, never identity nonmatches.
    pub(crate) fn prepare(
        vision: &ReconVision,
        frame: &CapturedFrame,
        sources: Vec<SourceCard>,
        capture: usize,
    ) -> Result<(Option<Self>, Vec<String>), String> {
        require(
            !sources.is_empty() && sources.len() <= 2,
            "invalid source-card count",
        )?;
        vision.verify_source_scene(frame)?;
        let mut ready = true;
        let mut reads = Vec::with_capacity(sources.len());
        for source in &sources {
            let actual = vision.source_identity(frame, source.location)?;
            ready &= actual == Some(source.card);
            let location = match source.location {
                PyramidTargetKind::Card { row, column } => format!("{row}|{column}"),
                PyramidTargetKind::Left => "DL".to_owned(),
                PyramidTargetKind::Right => "DR".to_owned(),
                _ => return Err("invalid source location".to_owned()),
            };
            reads.push(format!("{location}: expected {:?} of {:?}; observed {}",source.card.rank(),source.card.suit(),actual.map_or_else(|| "undetermined".to_owned(), |card| format!("{:?} of {:?}",card.rank(),card.suit()))));
        }
        Ok((
            ready.then_some(Self {
                sources,
                capture,
                confirmations: 0,
            }),
            reads,
        ))
    }


    /// Read the supported fresh POST confirmation count without granting authority.
    pub(crate) fn confirmations(&self) -> usize {
        self.confirmations
    }


    /// Read one existing fresh frame; a different identity or `None` qualifies.
    /// Reader/layout/scene errors remain errors and break consecutiveness.
    pub(crate) fn observe(
        &mut self,
        vision: &ReconVision,
        frame: &CapturedFrame,
        capture: usize,
    ) -> Result<bool, String> {
        if capture <= self.capture {
            self.reset();
            return Err("source disappearance requires a new capture serial".to_owned());
        }
        if capture != self.capture.saturating_add(1) {
            self.reset();
        }
        self.capture = capture;
        let result = (|| {
            vision.verify_source_scene(frame)?;
            self.sources.iter().try_fold(true, |gone, source| {
                Ok(gone & (vision.source_identity(frame, source.location)? != Some(source.card)))
            })
        })();
        match result {
            Ok(true) => self.confirmations = self.confirmations.saturating_add(1).min(2),
            Ok(false) => self.reset(),
            Err(error) => {
                self.reset();
                return Err(error);
            }
        }
        Ok(self.confirmations == 2)
    }


    /// Any unsupported or interrupted observation invalidates the successive pair.
    pub(crate) fn reset(&mut self) {
        self.confirmations = 0;
    }
}


impl ReconVision {
    /// Validate native layout and supported gameplay, without reading replacement cards.
    pub(crate) fn verify_source_scene(&self, frame: &CapturedFrame) -> Result<(), String> {
        require(
            frame.width == 1920 && frame.height == 1080,
            "source verification requires native 1920x1080",
        )?;
        let view = frame.as_view().map_err(|_| "invalid source frame layout")?;
        require(
            !self.dialog_matches(view, true)?,
            "source verification refused Undo All dialog",
        )?;
        let empty_stock = self.matches_execution_empty_stock(view, None, None)?
            || self.matches_native_empty_stock(view, None, None)?
            || self.source_exhausted_stock_scene(view)?;
        for (index, pattern) in self.board.scene.iter().enumerate() {
            if empty_stock && index == EXECUTION_EMPTY_STOCK_SCENE_INDEX {
                continue;
            }
            require(
                self.native_scene_anchor(view, index, pattern)?,
                &format!("source board scene[{index}] mismatch"),
            )?;
        }
        Ok(())
    }


    /// Select a D source only from a recognised DL card or positively empty DL.
    pub(crate) fn draw_source(&self, frame: &CapturedFrame) -> Result<PyramidTargetKind, String> {
        self.verify_source_scene(frame)?;
        let view = frame.as_view().map_err(|_| "invalid source frame layout")?;
        match self.source_draw_face(view)? {
            StockFace::Card(_) => Ok(PyramidTargetKind::Left),
            StockFace::Empty => {
                require(
                    self.source_identity(frame, PyramidTargetKind::Right)?
                        .is_some(),
                    "RECYCLE source RIGHT is not identified",
                )?;
                Ok(PyramidTargetKind::Right)
            }
        }
    }


    /// Use the existing rank/suit reader at precisely the original location.
    /// `None` is an undetermined identity, not a positive empty-pile assertion.
    pub(crate) fn source_identity(
        &self,
        frame: &CapturedFrame,
        location: PyramidTargetKind,
    ) -> Result<Option<Card>, String> {
        let view = frame.as_view().map_err(|_| "invalid source frame layout")?;
        let (reader, origin) = match location {
            PyramidTargetKind::Left => (&self.stock_reader, (LEFT_BOUNDS.x, LEFT_BOUNDS.y)),
            PyramidTargetKind::Right => (&self.stock_reader, (RIGHT_BOUNDS.x, RIGHT_BOUNDS.y)),
            PyramidTargetKind::Card { row, column } => {
                let index = xsar::games::pyramid::planning::tableau_index(
                    usize::from(row),
                    usize::from(column),
                )
                .ok_or("invalid source coordinate")?;
                (
                    &self.tableau_reader,
                    *TABLEAU_ORIGINS.get(index).ok_or("invalid source slot")?,
                )
            }
            PyramidTargetKind::Move => return Err("Move control is not a card source".to_owned()),
        };
        reader.recognise(view, origin)
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    /// Reuse byte-original captures as offline observations, never as live freshness proof.
    fn frame(path: &str) -> CapturedFrame {
        crate::capture::decode_png(
            &std::fs::read(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/fixtures")
                    .join(path),
            )
            .unwrap(),
        )
        .unwrap()
    }


    /// Put retained original glyphs into the supported after-scene for an offline source.
    /// The derivative is synthetic and makes no runtime pre-action freshness claim.
    fn supported_source(
        _vision: &ReconVision,
        before: CapturedFrame,
        after: &CapturedFrame,
    ) -> CapturedFrame {
        let mut supported = after.clone();
        for (x, y) in [TABLEAU_ORIGINS[22], (LEFT_BOUNDS.x, LEFT_BOUNDS.y)] {
            for row in y + 4..y + 4 + 49 {
                for column in x + 3..x + 3 + 31 {
                    let start = usize::try_from(row * before.width + column).unwrap() * 4;
                    supported.pixels[start..start + 4]
                        .copy_from_slice(&before.pixels[start..start + 4]);
                }
            }
        }
        supported
    }


    /// Both exact originals must be identified together; separate partial frames cannot authorise a pair.
    #[test]
    fn readiness_173_both_sources_must_match_the_same_supported_frame() {
        let vision = ReconVision::bundled_native_v23().unwrap();
        let after = frame("pyramid-recognition-168/green/last-observed.png");
        let before = supported_source(
            &vision,
            frame("pyramid-recognition-168/green/first-unmatched.png"),
            &after,
        );
        let sources = vec![
            SourceCard {
                location: PyramidTargetKind::Card { row: 7, column: 2 },
                card: Card::new(Rank::Seven, Suit::Spades),
            },
            SourceCard {
                location: PyramidTargetKind::Left,
                card: Card::new(Rank::Six, Suit::Hearts),
            },
        ];
        assert!(
            SourceTransition::prepare(&vision, &before, sources.clone(), 1)
                .unwrap()
                .0
                .is_some()
        );
        for (x, y) in [TABLEAU_ORIGINS[22], (LEFT_BOUNDS.x, LEFT_BOUNDS.y)] {
            let mut partial = after.clone();
            for row in y + 4..y + 53 {
                for column in x + 3..x + 34 {
                    let offset = usize::try_from(row * before.width + column).unwrap() * 4;
                    partial.pixels[offset..offset + 4]
                        .copy_from_slice(&before.pixels[offset..offset + 4]);
                }
            }
            let (ready, reads) =
                SourceTransition::prepare(&vision, &partial, sources.clone(), 2).unwrap();
            assert!(ready.is_none());
            assert_eq!(reads.len(), 2);
        }
        let mut invalid = before;
        invalid.width = 1;
        assert!(SourceTransition::prepare(&vision, &invalid, sources, 3).is_err());
    }


    /// Ancestor paper or a different card cannot veto disappearance of the original.
    #[test]
    fn source_172_green_original_disappears_twice_without_replacement_matching() {
        let vision = ReconVision::bundled_native_v23().unwrap();
        let after = frame("pyramid-recognition-168/green/last-observed.png");
        let before = supported_source(
            &vision,
            frame("pyramid-recognition-168/green/first-unmatched.png"),
            &after,
        );
        let source = SourceCard {
            location: PyramidTargetKind::Card { row: 7, column: 2 },
            card: Card::new(Rank::Seven, Suit::Spades),
        };
        let mut transition = SourceTransition::begin(&vision, &before, vec![source], 10).unwrap();
        assert_ne!(
            vision.source_identity(&after, source.location).unwrap(),
            Some(source.card)
        );
        assert!(!transition.observe(&vision, &after, 11).unwrap());
        assert!(transition.observe(&vision, &after, 12).unwrap());
    }


    /// An unchanged original, a reused serial or an invalid frame never supplies a pair.
    #[test]
    fn source_172_resets_on_original_reappearance_or_structural_error() {
        let vision = ReconVision::bundled_native_v23().unwrap();
        let after = frame("pyramid-recognition-168/green/last-observed.png");
        let before = supported_source(
            &vision,
            frame("pyramid-recognition-168/green/first-unmatched.png"),
            &after,
        );
        let source = SourceCard {
            location: PyramidTargetKind::Card { row: 7, column: 2 },
            card: Card::new(Rank::Seven, Suit::Spades),
        };
        let mut transition = SourceTransition::begin(&vision, &before, vec![source], 0).unwrap();
        assert!(!transition.observe(&vision, &after, 1).unwrap());
        assert!(!transition.observe(&vision, &before, 2).unwrap());
        assert!(!transition.observe(&vision, &after, 3).unwrap());
        assert!(transition.observe(&vision, &after, 3).is_err());
        assert!(!transition.observe(&vision, &after, 4).unwrap());
        let mut invalid = after.clone();
        invalid.width = 1;
        assert!(transition.observe(&vision, &invalid, 5).is_err());
        assert!(!transition.observe(&vision, &after, 6).unwrap());
        assert!(transition.observe(&vision, &after, 7).unwrap());
        let wrong = SourceCard {
            card: Card::new(Rank::King, Suit::Clubs),
            ..source
        };
        assert!(SourceTransition::begin(&vision, &before, vec![wrong], 0).is_err());
    }

    /// A positively recognised replacement is allowed without matching its expected identity.
    #[test]
    fn source_172_different_recognised_stock_card_qualifies() {
        let vision = ReconVision::bundled_native_v23().unwrap();
        let after = frame("pyramid-recognition-168/green/last-observed.png");
        let before = supported_source(
            &vision,
            frame("pyramid-recognition-168/green/first-unmatched.png"),
            &after,
        );
        let source = SourceCard {
            location: PyramidTargetKind::Left,
            card: Card::new(Rank::Six, Suit::Hearts),
        };
        assert_eq!(
            vision.source_identity(&after, source.location).unwrap(),
            Some(Card::new(Rank::Jack, Suit::Hearts))
        );
        let mut transition = SourceTransition::begin(&vision, &before, vec![source], 0).unwrap();
        assert!(!transition.observe(&vision, &after, 1).unwrap());
        assert!(transition.observe(&vision, &after, 2).unwrap());
    }

}
