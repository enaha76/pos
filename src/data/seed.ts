import type {
  Category,
  Modifier,
  Product,
  ReasonCode,
  Server,
  Settings,
  Shift,
  ShiftAssignment,
  TableSpot,
  Zone,
} from "@/types/domain";

/*
  Seed configuration. In the spec everything here is owner-managed data
  (§3). This is just the starting content so the station is usable on first run.
*/

export const seedSettings: Settings = {
  spot_label: "Table", // Q5 default; owner-editable, per-zone overridable
  currency_symbol: "$",
};

export const seedShifts: Shift[] = [
  { shift_id: "shift_am", name: "Morning", start_time: "07:00", end_time: "16:00" },
  { shift_id: "shift_pm", name: "Evening", start_time: "16:00", end_time: "01:00" },
];

// §2 examples: VIP1, VIP2, Outside — with the three table modes.
export const seedZones: Zone[] = [
  { zone_id: "zone_vip1", name: "VIP1", display_order: 1, table_mode: "fixed", active: true },
  { zone_id: "zone_vip2", name: "VIP2", display_order: 2, table_mode: "free", active: true },
  {
    zone_id: "zone_out",
    name: "Outside",
    display_order: 3,
    table_mode: "none",
    active: true,
    spot_label: "Spot", // demonstrates the per-zone label override (Q5)
  },
];

export const seedTableSpots: TableSpot[] = [
  { table_id: "t_v1_1", zone_id: "zone_vip1", label: "1", active: true },
  { table_id: "t_v1_2", zone_id: "zone_vip1", label: "2", active: true },
  { table_id: "t_v1_3", zone_id: "zone_vip1", label: "3", active: true },
  { table_id: "t_v1_4", zone_id: "zone_vip1", label: "4", active: true },
];

export const seedServers: Server[] = [
  { server_id: "srv_amina", name: "Amina", pin: "1111", active: true },
  { server_id: "srv_bilal", name: "Bilal", pin: "2222", active: true },
  { server_id: "srv_cara", name: "Cara", pin: "3333", active: true },
];

// Each category owns one accent (uiux-spec §2.2 category color rule).
export const seedCategories: Category[] = [
  { category_id: "cat_starters", name: "Starters", color: "blue", display_order: 1 },
  { category_id: "cat_mains", name: "Mains", color: "purple", display_order: 2 },
  { category_id: "cat_sides", name: "Sides", color: "amber", display_order: 3 },
  { category_id: "cat_drinks", name: "Drinks", color: "mint", display_order: 4 },
  { category_id: "cat_desserts", name: "Desserts", color: "pink", display_order: 5 },
];

export const seedProducts: Product[] = [
  // Starters
  { product_id: "p_bruschetta", name: "Bruschetta", category_id: "cat_starters", price: 850, active: true },
  { product_id: "p_soup", name: "Soup of the Day", category_id: "cat_starters", price: 700, active: true },
  { product_id: "p_calamari", name: "Calamari", category_id: "cat_starters", price: 1150, active: true },
  { product_id: "p_salad", name: "Garden Salad", category_id: "cat_starters", price: 900, active: true },
  // Mains
  { product_id: "p_ribeye", name: "Ribeye Steak", category_id: "cat_mains", price: 2800, active: true },
  { product_id: "p_salmon", name: "Grilled Salmon", category_id: "cat_mains", price: 2200, active: true },
  { product_id: "p_burger", name: "House Burger", category_id: "cat_mains", price: 1600, active: true },
  { product_id: "p_pasta", name: "Truffle Pasta", category_id: "cat_mains", price: 1900, active: true },
  { product_id: "p_risotto", name: "Mushroom Risotto", category_id: "cat_mains", price: 1750, active: true },
  { product_id: "p_chicken", name: "Roast Chicken", category_id: "cat_mains", price: 1850, active: true },
  // Sides
  { product_id: "p_fries", name: "Fries", category_id: "cat_sides", price: 500, active: true },
  { product_id: "p_veg", name: "Seasonal Veg", category_id: "cat_sides", price: 600, active: true },
  { product_id: "p_bread", name: "Bread Basket", category_id: "cat_sides", price: 400, active: true },
  // Drinks
  { product_id: "p_mojito", name: "Mojito", category_id: "cat_drinks", price: 1100, active: true },
  { product_id: "p_wine", name: "House Wine", category_id: "cat_drinks", price: 950, active: true },
  { product_id: "p_beer", name: "Draft Beer", category_id: "cat_drinks", price: 750, active: true },
  { product_id: "p_soda", name: "Soft Drink", category_id: "cat_drinks", price: 350, active: true },
  { product_id: "p_coffee", name: "Coffee", category_id: "cat_drinks", price: 400, active: true },
  { product_id: "p_water", name: "Sparkling Water", category_id: "cat_drinks", price: 300, active: true },
  // Desserts
  { product_id: "p_tiramisu", name: "Tiramisu", category_id: "cat_desserts", price: 850, active: true },
  { product_id: "p_cheesecake", name: "Cheesecake", category_id: "cat_desserts", price: 800, active: true },
  { product_id: "p_icecream", name: "Ice Cream", category_id: "cat_desserts", price: 600, active: true },
];

export const seedModifiers: Modifier[] = [
  { modifier_id: "m_rare", product_id: "p_ribeye", name: "Rare", price_delta: 0 },
  { modifier_id: "m_medium", product_id: "p_ribeye", name: "Medium", price_delta: 0 },
  { modifier_id: "m_welldone", product_id: "p_ribeye", name: "Well done", price_delta: 0 },
  { modifier_id: "m_cheese", product_id: "p_burger", name: "Extra cheese", price_delta: 150 },
  { modifier_id: "m_bacon", product_id: "p_burger", name: "Add bacon", price_delta: 250 },
];

// Owner-managed reason lists (§8). Separate kinds for void / comp / unpaid.
export const seedReasonCodes: ReasonCode[] = [
  { reason_id: "r_v_mistake", kind: "void", label: "Order mistake", active: true },
  { reason_id: "r_v_changed", kind: "void", label: "Customer changed mind", active: true },
  { reason_id: "r_v_duplicate", kind: "void", label: "Duplicate entry", active: true },
  { reason_id: "r_c_kitchen", kind: "comp", label: "Kitchen error", active: true },
  { reason_id: "r_c_dissatisfied", kind: "comp", label: "Customer dissatisfaction", active: true },
  { reason_id: "r_c_spill", kind: "comp", label: "Spill / accident", active: true },
  { reason_id: "r_c_staff", kind: "comp", label: "Staff meal", active: true },
  { reason_id: "r_u_walkout", kind: "unpaid", label: "Walkout", active: true },
  { reason_id: "r_u_writeoff", kind: "unpaid", label: "Manager write-off", active: true },
];

// Roster for a given day — note the same zone gets a different server per shift
// (cashier-charge §2: "the same zone may have a different server tomorrow").
export function seedShiftAssignments(date: string): ShiftAssignment[] {
  const rows: Array<[string, string, string]> = [
    // [server, zone, shift]
    ["srv_amina", "zone_vip1", "shift_am"],
    ["srv_bilal", "zone_vip2", "shift_am"],
    ["srv_cara", "zone_out", "shift_am"],
    ["srv_bilal", "zone_vip1", "shift_pm"],
    ["srv_cara", "zone_vip2", "shift_pm"],
    ["srv_amina", "zone_out", "shift_pm"],
  ];
  return rows.map(([server_id, zone_id, shift_id], i) => ({
    assignment_id: `asn_seed_${i}`,
    server_id,
    zone_id,
    shift_id,
    date,
  }));
}

export const PAYMENT_METHODS = ["Espèces", "Portefeuille"] as const;
