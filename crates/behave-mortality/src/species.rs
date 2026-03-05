//! Species master table for tree mortality.
//!
//! Maps FOFEM species codes to mortality equations, bark equations,
//! canopy coefficient codes, GACC region availability, equation types,
//! and crown damage equation codes.
//!
//! C++ source: species_master_table.h / species_master_table.cpp

use crate::equations::{CrownDamageEquationCode, EquationType};

/// Geographic Area Coordination Center regions.
///
/// C++ source: `GACC` in species_master_table.h
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i8)]
pub enum Gacc {
    NotSet = -1,
    Alaska = 1,
    California = 2,
    EasternArea = 3,
    GreatBasin = 4,
    NorthernRockies = 5,
    Northwest = 6,
    RockyMountain = 7,
    SouthernArea = 8,
    Southwest = 9,
}

/// A single species record from the master table.
///
/// C++ struct: `SpeciesMasterTableRecord`
#[derive(Debug, Clone)]
pub struct SpeciesMasterTableRecord {
    pub species_code: String,
    pub scientific_name: String,
    pub common_name: String,
    pub mortality_equation_number: i32,
    pub bark_equation_number: i32,
    pub crown_coefficient_code: i32,
    /// GACC region flags: -1 = not available, positive = GACC enum value
    pub regions: [i8; 9], // [Alaska, California, EasternArea, GreatBasin, NorthernRockies, Northwest, RockyMountain, SouthernArea, Southwest]
    pub equation_type: EquationType,
    pub crown_damage_equation_code: CrownDamageEquationCode,
}

/// Lookup table of species → mortality equation mappings.
///
/// C++ class: `SpeciesMasterTable`
#[derive(Debug, Clone)]
pub struct SpeciesMasterTable {
    pub records: Vec<SpeciesMasterTableRecord>,
}

impl SpeciesMasterTable {
    pub fn new() -> Self {
        let mut table = Self { records: Vec::new() };
        table.initialize();
        table
    }

    pub fn record_count(&self) -> usize {
        self.records.len()
    }

    /// Find index by species code (case-insensitive, first match).
    /// Returns -1 if not found (matching C++ behavior).
    pub fn index_from_species_code(&self, species_code: &str) -> i32 {
        let upper = species_code.to_uppercase();
        for (i, r) in self.records.iter().enumerate() {
            if r.species_code.is_empty() {
                break;
            }
            if r.species_code == upper {
                return i as i32;
            }
        }
        -1
    }

    /// Find index by species code AND equation type (case-insensitive).
    /// Returns -1 if not found (matching C++ behavior).
    pub fn index_from_species_code_and_equation_type(
        &self,
        species_code: &str,
        equation_type: EquationType,
    ) -> i32 {
        let upper = species_code.to_uppercase();
        for (i, r) in self.records.iter().enumerate() {
            if r.species_code.is_empty() {
                break;
            }
            if r.species_code == upper && r.equation_type == equation_type {
                return i as i32;
            }
        }
        -1
    }

    /// Check if a species is available in a given GACC region.
    pub fn is_available_in_region(&self, index: usize, gacc: Gacc) -> bool {
        if index >= self.records.len() {
            return false;
        }
        let region_idx = match gacc {
            Gacc::Alaska => 0,
            Gacc::California => 1,
            Gacc::EasternArea => 2,
            Gacc::GreatBasin => 3,
            Gacc::NorthernRockies => 4,
            Gacc::Northwest => 5,
            Gacc::RockyMountain => 6,
            Gacc::SouthernArea => 7,
            Gacc::Southwest => 8,
            Gacc::NotSet => return false,
        };
        self.records[index].regions[region_idx] > 0
    }

    // --- Internal ---

    fn add_record(
        &mut self,
        species_code: &str,
        scientific_name: &str,
        common_name: &str,
        mort: i32,
        brk: i32,
        crn: i32,
        alaska: i8,
        california: i8,
        eastern: i8,
        great_basin: i8,
        northern_rockies: i8,
        northwest: i8,
        rocky_mountain: i8,
        southern: i8,
        southwest: i8,
        equation_type: EquationType,
        crown_damage_code: CrownDamageEquationCode,
    ) {
        self.records.push(SpeciesMasterTableRecord {
            species_code: species_code.to_string(),
            scientific_name: scientific_name.to_string(),
            common_name: common_name.to_string(),
            mortality_equation_number: mort,
            bark_equation_number: brk,
            crown_coefficient_code: crn,
            regions: [alaska, california, eastern, great_basin, northern_rockies, northwest, rocky_mountain, southern, southwest],
            equation_type,
            crown_damage_equation_code: crown_damage_code,
        });
    }

    fn initialize(&mut self) {
        use CrownDamageEquationCode as CD;
        use EquationType as ET;

        //                                species  scientific_name           common_name                                              mort brk crn AK CA EA GB NR NW RM SA SW equation_type          crown_damage
        self.add_record("ABAM",  "Abies amabilis",              "Pacific silver fir",                              1, 26,  1,  1, -1, -1, -1, -1,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("ABBA",  "Abies balsamea",              "balsam fir",                                      1, 10,  2, -1, -1,  3, -1, -1, -1, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("ABCO",  "Abies concolor",              "white fir",                                      10, 27,  2, -1,  2,  3,  4, -1,  6,  7, -1,  9, ET::CrownScorch, CD::NotSet);
        self.add_record("ABGR",  "Abies grandis",               "grand fir",                                      11, 25,  3, -1,  2, -1,  4,  5,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("ABLA",  "Abies lasiocarpa",            "corkbark fir",                                   11, 20,  4,  1,  2, -1,  4,  5,  6,  7, -1,  9, ET::CrownScorch, CD::NotSet);
        self.add_record("ABLA",  "Abies lasiocarpa",            "subalpine fir",                                  11, 20,  4,  1,  2, -1,  4,  5,  6,  7, -1,  9, ET::CrownScorch, CD::NotSet);
        self.add_record("ABMA",  "Abies magnifica",             "California red fir",                             16, 18,  5, -1,  2, -1,  4, -1,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("ABPR",  "Abies procera",               "noble fir",                                       1, 24,  7, -1,  2, -1, -1, -1,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("ACBA3", "Acer barbatum",               "Florida maple",                                   1,  8, 21, -1, -1, -1, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("ACMA3", "Acer macrophyllum",           "bigleaf maple",                                   1,  3, 21, -1,  2, -1, -1, -1,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("ACNE2", "Acer negundo",                "boxelder",                                        1, 13, 21, -1,  2,  3,  4,  5,  6,  7,  8,  9, ET::CrownScorch, CD::NotSet);
        self.add_record("ACNI5", "Acer nigrum",                 "black maple",                                     1, 14, 21, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("ACPE",  "Acer pensylvanicum",          "striped maple",                                   1, 24, 21, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("ACRU",  "Acer rubrum",                 "red maple",                                     100,  7, 21, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::BoleChar,    CD::NotSet);
        self.add_record("ACSA2", "Acer saccharinum",            "silver maple",                                    1, 10, 21, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("ACSA3", "Acer saccharum",              "sugar maple",                                     1, 12, 21, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("ACSP2", "Acer spicatum",               "mountain maple",                                  1, 19, 21, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("AEFL",  "Aesculus flava",              "yellow buckeye",                                  1, 29, 39, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("AEGL",  "Aesculus glabra",             "Ohio buckeye",                                    1, 15, 39, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("AIAL",  "Ailanthus altissima",         "ailanthus",                                       1, 29, 39, -1,  2,  3, -1, -1,  6,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("ALRH2", "Alnus rhombifolia",           "white alder",                                     1,  1, 23, -1,  2, -1, -1, -1,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("ALRU2", "Alnus rubra",                 "red alder",                                       1,  1, 22,  1,  2,  3, -1,  5,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("AMAR3", "Amelanchier arborea",         "common serviceberry",                             1, 29, 39, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("ARME",  "Arbutus menziesii",           "Pacific madrone",                                 1, 34, 39, -1,  2, -1, -1, -1,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("BEAL2", "Betula alleghaniensis",       "yellow birch",                                    1, 10, 24, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("BELE",  "Betula lenta",                "sweet birch",                                     1,  9, 24, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("BENI",  "Betula nigra",                "river birch",                                     1,  8, 24, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("BEOC2", "Betula occidentalis",         "water birch",                                     1,  1, 24, -1,  2, -1,  4,  5,  6,  7, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("BEPA",  "Betula papyrifera",           "paper birch",                                     1,  1, 24,  1, -1,  3, -1,  5,  6,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("BETSPP","Betula species",              "birch species",                                   1,  1, 24, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("CADE27","Calocedrus decurrens",        "incense-cedar",                                  12, 34, 18, -1,  2, -1,  4, -1,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("CACA18","Carpinus caroliniana",        "American hornbeam, musclewood",                   1,  9, 39, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("CAAL27","Carya alba",                  "mockernut hickory",                               1, 22, 39, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("CAAQ",  "Carya aquatica",              "water hickory",                                   1, 19,  0, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("CACOL3","Carya cordiformis",           "bitternut hickory",                               1, 16, 39, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("CAGL8", "Carya glabra",                "pignut hickory",                                  1, 16, 39, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("CAIL2", "Carya illinoinensis",         "pecan",                                           1, 15, 39, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("CALA21","Carya laciniosa",             "shellbark hickory",                               1, 22, 39, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("CAOV2", "Carya ovata",                 "shagbark hickory",                                1, 19, 39, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("CARSPP","Carya species",               "hickory species",                                 1, 23, 39, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("CATE9", "Carya texana",                "black hickory",                                   1, 19, 39, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("CADE12","Castanea dentata",            "American chestnut",                               1, 19, 39, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("CELA",  "Celtis laevigata",            "sugarberry",                                      1, 15, 39, -1, -1,  3, -1, -1,  6,  7,  8,  9, ET::CrownScorch, CD::NotSet);
        self.add_record("CELA",  "Celtis laevigata",            "netleaf hackberry",                               1, 15, 39, -1, -1,  3, -1, -1,  6,  7,  8,  9, ET::CrownScorch, CD::NotSet);
        self.add_record("CEOC",  "Celtis occidentalis",         "hackberry",                                       1, 14, 24, -1, -1,  3, -1,  5, -1,  7,  8,  9, ET::CrownScorch, CD::NotSet);
        self.add_record("CECA4", "Cercis canadensis",           "eastern redbud",                                  1, 14, 39, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("CHLA",  "Chamaecyparis lawsoniana",    "Port-Orford-cedar",                               1, 39,  9, -1,  2, -1, -1, -1,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("CHNO",  "Chamaecyparis nootkatensis",  "Alaska yellow-cedar",                             1,  2,  9,  1, -1, -1, -1, -1,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("CHTH2", "Chamaecyparis thyoides",      "Atlantic white-cedar",                            1,  4,  9, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("CHCHC4","Chrysolepis chrysophylla",    "giant chinkapin, golden chinkapin",               1, 24, 25, -1,  2, -1, -1, -1,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("COFL2", "Cornus florida",              "flowering dogwood",                             101, 20, 34, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::BoleChar,    CD::NotSet);
        self.add_record("CONU4", "Cornus nuttallii",            "Pacific dogwood",                                 1, 35, 34, -1,  2, -1, -1, -1,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("CRASPP","Crataegus species",           "hawthorn species",                                1, 35, 35, -1, -1,  3, -1,  5, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("DIVI5", "Diospyros virginiana",        "common persimmon",                                1, 20, 39, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("FAGR",  "Fagus grandifolia",           "American beech",                                  1,  4, 39, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("FRAM2", "Fraxinus americana",          "white ash",                                       1, 21, 39, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("FRNI",  "Fraxinus nigra",              "black ash",                                       1, 14, 39, -1, -1,  3, -1,  5, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("FRPE",  "Fraxinus pennsylvanica",      "green ash",                                       1, 18, 39, -1, -1,  3, -1,  5, -1,  7,  8,  9, ET::CrownScorch, CD::NotSet);
        self.add_record("FRPR",  "Fraxinus profunda",           "pumpkin ash",                                     1, 16, 39, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("FRQU",  "Fraxinus quadrangulata",      "blue ash",                                        1,  9, 39, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("FRASPP","Fraxinus species",            "ash species",                                     1, 21, 39, -1,  2,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("GLTR",  "Gleditsia triacanthos",       "honeylocust",                                     1, 17, 39, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("GOLA",  "Gordonia lasianthus",         "loblolly-bay",                                    1, 17, 39, -1, -1, -1, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("GYDI",  "Gymnocladus dioicus",         "Kentucky coffeetree",                             1, 10, 39, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("HALSPP","Halesia species",             "silverbell species",                              1, 17, 39, -1, -1, -1, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("ILOP",  "Ilex opaca",                  "American holly",                                  1, 21, 39, -1,  2,  3, -1, -1,  6, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("JUCI",  "Juglans cinerea",             "butternut",                                       1, 20, 39, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("JUNI",  "Juglans nigra",               "black walnut",                                    1, 20, 39, -1, -1,  3, -1, -1, -1,  7,  8,  9, ET::CrownScorch, CD::NotSet);
        self.add_record("JUOC",  "Juniperus occidentalis",      "western juniper",                                 1, 24, 29, -1,  2, -1,  4, -1,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("JUVI",  "Juniperus virginiana",        "southern redcedar",                               1, 19, 18, -1, -1,  3, -1,  5, -1,  7,  8,  9, ET::CrownScorch, CD::NotSet);
        self.add_record("JUVI",  "Juniperus virginiana",        "eastern redcedar",                                1, 19, 18, -1, -1,  3, -1,  5, -1,  7,  8,  9, ET::CrownScorch, CD::NotSet);
        self.add_record("LALA",  "Larix laricina",              "tamarack",                                        1, 10, 14,  1, -1,  3, -1, -1, -1, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("LALY",  "Larix lyallii",               "subalpine larch",                                 1, 29,  3, -1, -1, -1, -1,  5,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("LAOC",  "Larix occidentalis",          "western larch",                                  14, 36, 14, -1,  2, -1,  4,  5,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("LIST2", "Liquidambar styraciflua",     "sweetgum",                                        1, 34, 39, -1,  2,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("LITU",  "Liriodendron tulipifera",     "yellow-poplar",                                   1, 29, 39, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("LIDE3", "Lithocarpus densiflorus",     "tanoak",                                          1, 30, 39, -1,  2, -1, -1, -1,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("MAPO",  "Maclura pomifera",            "Osage-orange",                                    1, 16, 39, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("MAAC",  "Magnolia acuminata",          "cucumbertree",                                    1, 15, 39, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("MAGR4", "Magnolia grandiflora",        "southern magnolia",                               1, 12, 39, -1, -1, -1, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("MAMA2", "Magnolia macrophylla",        "bigleaf magnolia",                                1, 12, 39, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("MAGSPP","Magnolia species",            "magnolia species",                                1, 18, 39, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("MAVI2", "Magnolia virginiana",         "sweetbay",                                        1, 19, 39, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("MALSPP","Malus species",               "apple species",                                   1, 22, 39, -1, -1,  3, -1,  5, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("MOAL",  "Morus alba",                  "white mulberry",                                  1, 17, 39, -1,  2,  3, -1, -1, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("MORU2", "Morus rubra",                 "red mulberry",                                    1, 17, 39, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("MORSPP","Morus species",               "mulberry species",                                1, 12, 39, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("NYAQ2", "Nyssa aquatica",              "water tupelo",                                    1, 32, 39, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("NYBI",  "Nyssa biflora",               "swamp tupelo",                                    1, 32, 39, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("NYOG",  "Nyssa ogeche",                "Ogeechee tupelo",                                 1, 32, 39, -1, -1, -1, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("NYSY",  "Nyssa sylvatica",             "blackgum",                                      102, 32, 39, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::BoleChar,    CD::NotSet);
        self.add_record("OSVI",  "Ostrya virginiana",           "eastern hophornbeam",                             1, 16, 39, -1, -1,  3, -1,  5, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("OXAR",  "Oxydendrum arboreum",         "sourwood",                                      103, 15, 39, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::BoleChar,    CD::NotSet);
        self.add_record("PATO2", "Paulownia tomentosa",         "paulownia, empress-tree",                         1, 29, 39, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PEBO",  "Persea borbonia",             "redbay",                                          1, 17, 39, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PIAB",  "Picea abies",                 "Norway spruce",                                   3,  8,  1, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PIEN",  "Picea engelmannii",           "Engelmann spruce",                               15, 15,  1, -1,  2, -1,  4,  5,  6,  7, -1,  9, ET::CrownScorch, CD::NotSet);
        self.add_record("PIGL",  "Picea glauca",                "white spruce",                                    3,  4,  1,  1, -1,  3, -1,  5, -1,  7, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PIMA",  "Picea mariana",               "black spruce",                                    3, 11,  1,  1, -1,  3, -1, -1, -1, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PIPU",  "Picea pungens",               "blue spruce",                                     3, 10,  1, -1, -1,  3,  4,  5, -1,  7, -1,  9, ET::CrownScorch, CD::NotSet);
        self.add_record("PIRU",  "Picea rubens",                "red spruce",                                      3, 13,  1, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PISI",  "Picea sitchensis",            "Sitka spruce",                                    3,  6,  1,  1,  2, -1, -1, -1,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PIAL",  "Pinus albicaulis",            "whitebark pine",                                 17,  9, 31, -1,  2, -1,  4,  5,  6,  7, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PIAT",  "Pinus attenuata",             "knobcone pine",                                   1,  9, 32, -1,  2, -1, -1, -1,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PIBA2", "Pinus banksiana",             "jack pine",                                       1, 19, 11, -1, -1,  3, -1, -1, -1,  7, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PICL",  "Pinus clausa",                "sand pine",                                       1, 14, 11, -1, -1, -1, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PICO",  "Pinus contorta",              "lodgepole pine",                                 17,  7, 11,  1,  2, -1,  4,  5,  6,  7, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PIEC2", "Pinus echinata",              "shortleaf pine",                                  1, 16, 15, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PIEL",  "Pinus elliottii",             "slash pine",                                      1, 31, 15, -1, -1, -1, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PIFL2", "Pinus flexilis",              "limber pine",                                     1,  9, 31, -1,  2, -1,  4,  5, -1,  7, -1,  9, ET::CrownScorch, CD::NotSet);
        self.add_record("PIGL2", "Pinus glabra",                "spruce pine",                                     1, 14, 11, -1, -1, -1, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PIJE",  "Pinus jeffreyi",              "Jeffrey pine",                                   19, 37, 12, -1,  2, -1,  4, -1,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PILA",  "Pinus lambertiana",           "sugar pine",                                     18, 38, 13, -1,  2, -1, -1, -1,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PIMO3", "Pinus monticola",             "western white pine",                              1, 14, 14, -1,  2, -1,  4,  5,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PIPA2", "Pinus palustris",             "longleaf pine",                                   5,100, 15, -1, -1, -1, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PIPO",  "Pinus ponderosa",             "ponderosa pine",                                 19, 36, 15, -1,  2,  3,  4,  5,  6,  7,  8,  9, ET::CrownScorch, CD::NotSet);
        self.add_record("PIPU5", "Pinus pungens",               "Table Mountain pine",                             1, 19, 11, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PIRE",  "Pinus resinosa",              "red pine",                                        1, 22, 11, -1, -1,  3, -1, -1, -1,  7, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PIRI",  "Pinus rigida",                "pitch pine",                                      1, 24, 11, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PISA2", "Pinus sabiniana",             "gray or California foothill pine",                1, 12, 15, -1,  2, -1, -1, -1, -1, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PISE",  "Pinus serotina",              "pond pine",                                       1, 35, 11, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PIST",  "Pinus strobus",               "eastern white pine",                              1, 24, 14, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PISY",  "Pinus sylvestris",            "Scotch pine",                                     1,  9, 11, -1, -1,  3, -1,  5,  6,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PITA",  "Pinus taeda",                 "loblolly pine",                                   1, 30, 15, -1,  2,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PIVI2", "Pinus virginiana",            "Virginia pine",                                   1, 12, 11, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PLOC",  "Platanus occidentalis",       "American sycamore",                               1, 12, 39, -1,  2,  3, -1, -1, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("POBA2", "Populus balsamifera",          "balsam poplar",                                   1, 19, 27,  1,  2,  3,  4,  5,  6,  7, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("POBA2", "Populus balsamifera",          "black cottonwood",                                1, 19, 27,  1,  2,  3,  4,  5,  6,  7, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PODE",  "Populus deltoides",            "eastern cottonwood",                              1, 19,  0, -1, -1,  3, -1,  5, -1,  7,  8,  9, ET::CrownScorch, CD::NotSet);
        self.add_record("PODE",  "Populus deltoides",            "plains cottonwood",                               1, 19,  0, -1, -1,  3, -1,  5, -1,  7,  8,  9, ET::CrownScorch, CD::NotSet);
        self.add_record("POGR4", "Populus grandidentata",        "bigtooth aspen",                                  1, 18, 26, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("POHE4", "Populus heterophylla",         "swamp cottonwood",                                1, 29, 27, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("POTR12","Populus tremuloides",          "quaking aspen",                                   4, 23, 26,  1,  2,  3,  4,  5,  6,  7,  8,  9, ET::CrownScorch, CD::NotSet);
        self.add_record("PRAM",  "Prunus americana",             "American plum",                                   1, 19, 39, -1, -1,  3, -1,  5, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PREM",  "Prunus emarginata",            "bitter cherry",                                   1, 35, 36, -1,  2, -1, -1,  5,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PRPE2", "Prunus pensylvanica",          "pin cherry",                                      1, 24, 36, -1, -1,  3, -1,  5, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PRSE2", "Prunus serotina",              "black cherry",                                    1,  9, 36, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("MALPRU","Prunus species",               "cherry and plum species",                         1, 17, 39, -1, -1,  3, -1, -1,  6,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PRVI",  "Prunus virginiana",            "chokecherry",                                     1, 19, 36, -1,  2,  3,  4,  5,  6,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("PSME",  "Pseudotsuga menziesii",       "Douglas-fir",                                    20, 36, 16, -1,  2,  3,  4,  5,  6,  7, -1,  9, ET::CrownScorch, CD::NotSet);
        self.add_record("QUAG",  "Quercus agrifolia",            "California live oak",                             1, 29, 28, -1,  2, -1, -1, -1,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("QUAL",  "Quercus alba",                 "white oak",                                     104, 19, 28, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::BoleChar,    CD::NotSet);
        self.add_record("QUBI",  "Quercus bicolor",              "swamp white oak",                                 1, 24, 28, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("QUCH2", "Quercus chrysolepis",          "canyon live oak",                                 1,  3, 28, -1,  2, -1, -1, -1,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("QUCO2", "Quercus coccinea",             "scarlet oak",                                   105, 19, 28, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::BoleChar,    CD::NotSet);
        self.add_record("QUDU",  "Quercus douglasii",            "blue oak",                                        1, 12, 28, -1,  2, -1, -1, -1, -1, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("QUEL",  "Quercus ellipsoidalis",        "northern pin oak",                                1, 17, 28, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("QUFA",  "Quercus falcata",              "southern red oak",                                1, 23, 28, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("QUGA4", "Quercus garryana",             "Oregon white oak",                                1,  8, 28, -1,  2, -1, -1, -1,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("QUIM",  "Quercus imbricaria",           "shingle oak",                                     1, 20, 28, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("QUIN",  "Quercus incana",               "bluejack oak",                                    1, 17, 28, -1, -1, -1, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("QUKE",  "Quercus kelloggii",            "California black oak",                            1,  9, 28, -1,  2, -1, -1, -1,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("QULA2", "Quercus laevis",               "turkey oak",                                      1, 16, 28, -1, -1, -1, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("QULA3", "Quercus laurifolia",           "laurel oak",                                      1, 15, 28, -1, -1, -1, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("QULO",  "Quercus lobata",               "California white oak",                            1, 22, 28, -1,  2, -1, -1, -1, -1, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("QULY",  "Quercus lyrata",               "overcup oak",                                     1, 18, 28, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("QUMA2", "Quercus macrocarpa",           "bur oak",                                         1, 21, 28, -1, -1,  3, -1,  5, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("QUMA3", "Quercus marilandica",          "blackjack oak",                                 106, 16, 28, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::BoleChar,    CD::NotSet);
        self.add_record("QUMI",  "Quercus michauxii",            "swamp chestnut oak",                              1, 25, 28, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("QUEMON","Quercus montana",              "chestnut oak",                                  107, 28, 28, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::BoleChar,    CD::NotSet);
        self.add_record("QUMU",  "Quercus muehlenbergii",       "chinkapin oak",                                   1, 21, 28, -1, -1,  3, -1, -1, -1,  7,  8,  9, ET::CrownScorch, CD::NotSet);
        self.add_record("QUNI",  "Quercus nigra",                "water oak",                                       1, 15, 28, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("QUPA2", "Quercus palustris",            "pin oak",                                         1, 20, 28, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("QUPH",  "Quercus phellos",              "willow oak",                                      1, 20, 28, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("QURU",  "Quercus rubra",                "northern red oak",                                1, 21, 28, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("QUSH",  "Quercus shumardii",            "Shumard oak",                                     1, 16, 28, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("QUESPP","Quercus species",              "oak species",                                     1, 24, 28, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("QUST",  "Quercus stellata",             "post oak",                                        1, 23, 28, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("QUTE",  "Quercus texana",               "Nuttall oak",                                     1,  9, 28, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("QUVE",  "Quercus velutina",             "black oak",                                     108, 24, 28, -1, -1,  3, -1, -1, -1,  7,  8, -1, ET::BoleChar,    CD::NotSet);
        self.add_record("QUVI",  "Quercus virginiana",           "live oak",                                        1, 22, 28, -1, -1, -1, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("QUWI2", "Quercus wislizeni",            "interior live oak",                               1, 13, 28, -1,  2, -1, -1, -1, -1, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("ROPS",  "Robinia pseudoacacia",         "black locust",                                    1, 28, 39, -1, -1,  3, -1, -1,  6,  7,  8,  9, ET::CrownScorch, CD::NotSet);
        self.add_record("SABE2", "Salix bebbiana",               "Bebb willow",                                     1, 19, 37, -1, -1,  3, -1, -1, -1, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("SANI",  "Salix nigra",                  "black willow",                                    1, 19, 37, -1,  2,  3, -1,  5,  6,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("SALSPP","Salix species",                "willow species",                                  1, 20, 37, -1,  2,  3, -1,  5,  6,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("SAAL5", "Sassafras albidum",            "sassafras",                                     109, 14, 39, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::BoleChar,    CD::NotSet);
        self.add_record("SESE",  "Sequoia sempervirens",         "redwood",                                         1, 39,  0, -1,  2, -1, -1, -1,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("SOAM3", "Sorbus americana",             "American mountain-ash",                           1, 19, 39, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("TAAS",  "Taxodium ascendens",           "pondcypress",                                     1, 21, 39, -1, -1, -1, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("TADI2", "Taxodium distichum",           "baldcypress",                                     1,  4, 39, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("TABR2", "Taxus brevifolia",             "Pacific yew",                                     1,  4, 33,  1,  2, -1, -1,  5,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("THOC2", "Thuja occidentalis",           "northern white-cedar",                            1,  4, 18, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("THPL",  "Thuja plicata",                "western redcedar",                                1, 14, 18,  1,  2, -1, -1,  5,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("TIAM",  "Tilia americana",              "American basswood",                               1, 17, 39, -1, -1,  3, -1,  5, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("TIAM",  "Tilia americana",              "white basswood",                                  1, 17, 39, -1, -1,  3, -1,  5, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("TIAM",  "Tilia americana",              "Carolina basswood",                               1, 17, 39, -1, -1,  3, -1,  5, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("TSCA",  "Tsuga canadensis",             "eastern hemlock",                                 1, 18, 19, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("TSHE",  "Tsuga heterophylla",           "western hemlock",                                 1, 19, 19,  1,  2, -1, -1,  5,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("TSME",  "Tsuga mertensiana",            "mountain hemlock",                                1, 19,  2,  1,  2, -1,  4,  5,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("ULAL",  "Ulmus alata",                  "winged elm",                                      1, 10, 39, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("ULAM",  "Ulmus americana",              "American elm",                                    1, 10, 39, -1, -1,  3, -1,  5, -1,  7,  8,  9, ET::CrownScorch, CD::NotSet);
        self.add_record("ULPU",  "Ulmus pumila",                 "Siberian elm",                                    1, 17, 39, -1, -1,  3,  4,  5, -1,  7,  8,  9, ET::CrownScorch, CD::NotSet);
        self.add_record("ULRU",  "Ulmus rubra",                  "slippery elm",                                    1, 11, 39, -1, -1,  3, -1,  5, -1,  7,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("ULMSPP","Ulmus species",                "elm species",                                     1, 18, 39, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("ULTH",  "Ulmus thomasii",               "rock elm",                                        1, 12, 39, -1, -1,  3, -1, -1, -1, -1,  8, -1, ET::CrownScorch, CD::NotSet);
        self.add_record("UMCA",  "Umbellularia californica",     "California laurel",                               1,  5, 39, -1,  2, -1, -1, -1,  6, -1, -1, -1, ET::CrownScorch, CD::NotSet);
    }
}

impl Default for SpeciesMasterTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> SpeciesMasterTable {
        SpeciesMasterTable::new()
    }

    #[test]
    fn total_record_count() {
        // C++ has 197 species records
        let t = table();
        assert_eq!(t.record_count(), 197);
    }

    #[test]
    fn lookup_abam() {
        let t = table();
        let idx = t.index_from_species_code("ABAM");
        assert_eq!(idx, 0);
        let r = &t.records[idx as usize];
        assert_eq!(r.scientific_name, "Abies amabilis");
        assert_eq!(r.common_name, "Pacific silver fir");
        assert_eq!(r.mortality_equation_number, 1);
        assert_eq!(r.bark_equation_number, 26);
        assert_eq!(r.crown_coefficient_code, 1);
        assert_eq!(r.equation_type, EquationType::CrownScorch);
    }

    #[test]
    fn lookup_pipo() {
        let t = table();
        let idx = t.index_from_species_code("PIPO");
        assert!(idx >= 0);
        let r = &t.records[idx as usize];
        assert_eq!(r.common_name, "ponderosa pine");
        assert_eq!(r.mortality_equation_number, 19);
        assert_eq!(r.bark_equation_number, 36);
        // Available in all 9 GACC regions except Alaska
        assert!(!t.is_available_in_region(idx as usize, Gacc::Alaska));
        assert!(t.is_available_in_region(idx as usize, Gacc::California));
        assert!(t.is_available_in_region(idx as usize, Gacc::Southwest));
    }

    #[test]
    fn lookup_psme_douglas_fir() {
        let t = table();
        let idx = t.index_from_species_code("PSME");
        assert!(idx >= 0);
        let r = &t.records[idx as usize];
        assert_eq!(r.common_name, "Douglas-fir");
        assert_eq!(r.mortality_equation_number, 20);
        assert_eq!(r.crown_coefficient_code, 16);
    }

    #[test]
    fn lookup_acru_bole_char() {
        let t = table();
        let idx = t.index_from_species_code("ACRU");
        assert!(idx >= 0);
        let r = &t.records[idx as usize];
        assert_eq!(r.common_name, "red maple");
        assert_eq!(r.equation_type, EquationType::BoleChar);
        assert_eq!(r.mortality_equation_number, 100);
    }

    #[test]
    fn case_insensitive_lookup() {
        let t = table();
        assert_eq!(
            t.index_from_species_code("abam"),
            t.index_from_species_code("ABAM")
        );
    }

    #[test]
    fn unknown_species_returns_negative() {
        let t = table();
        assert_eq!(t.index_from_species_code("XXXX"), -1);
    }

    #[test]
    fn duplicate_species_codes_exist() {
        // Some species have multiple entries (e.g., ABLA for corkbark fir and subalpine fir)
        let t = table();
        let idx1 = t.index_from_species_code("ABLA");
        assert!(idx1 >= 0);
        // The first match is corkbark fir
        assert_eq!(t.records[idx1 as usize].common_name, "corkbark fir");
    }

    #[test]
    fn lookup_by_code_and_equation_type() {
        let t = table();
        let idx = t.index_from_species_code_and_equation_type("ACRU", EquationType::BoleChar);
        assert!(idx >= 0);
        assert_eq!(t.records[idx as usize].common_name, "red maple");

        // ACRU with CrownScorch should not match
        let idx2 = t.index_from_species_code_and_equation_type("ACRU", EquationType::CrownScorch);
        assert_eq!(idx2, -1);
    }

    #[test]
    fn last_record_is_umca() {
        let t = table();
        let last = t.records.last().unwrap();
        assert_eq!(last.species_code, "UMCA");
        assert_eq!(last.common_name, "California laurel");
    }

    #[test]
    fn quaking_aspen_all_regions() {
        let t = table();
        let idx = t.index_from_species_code("POTR12");
        assert!(idx >= 0);
        let r = &t.records[idx as usize];
        assert_eq!(r.common_name, "quaking aspen");
        // Available in all 9 GACC regions
        for gacc in [Gacc::Alaska, Gacc::California, Gacc::EasternArea, Gacc::GreatBasin,
                     Gacc::NorthernRockies, Gacc::Northwest, Gacc::RockyMountain,
                     Gacc::SouthernArea, Gacc::Southwest] {
            assert!(t.is_available_in_region(idx as usize, gacc), "POTR12 should be in {:?}", gacc);
        }
    }
}
