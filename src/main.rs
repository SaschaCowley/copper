use libcopper::make_table;

make_table! {
	Unit::Metre -> Unit::Yard => div 0.9144,
	Unit::Metre -> Unit::Centimetre => mul 100,
	unit::Inch -> {
		Unit::Foot => mul 12,
		Unit::Yard => mul 36,
	}
}
