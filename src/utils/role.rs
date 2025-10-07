use rand::seq::{IndexedRandom, SliceRandom};
use uuid::Uuid;
use crate::dto::{RoleDistribution, RoleResponse, RolesByType};
use crate::models::RoleType;

pub fn group_roles_by_type(
    roles: Vec<RoleResponse>,
) -> RolesByType {
    let mut result = RolesByType {
        beasts: Vec::new(),
        citizens: Vec::new(),
        special: Vec::new(),
    };

    for role in roles {
        match role.role_type {
            RoleType::Beast => result.beasts.push(role),
            RoleType::Citizen => {
                if role.slug.to_lowercase() == "villager" {
                    result.citizens.push(role);
                } else {
                    result.special.push(role);
                }
            },
            RoleType::Neutral => result.special.push(role),
        }
    }

    result
}

pub fn select_roles_for_game (
    roles: RolesByType,
    distribution: RoleDistribution,
) -> Result<Vec<Uuid>, String>{
    let mut selected_roles = Vec::new();
    let mut rng = rand::rng();

    if  roles.beasts.is_empty() {
        return Err("No beasts available".to_string());
    }
    let werewolf = roles.beasts.first().unwrap();
    for _ in 0..distribution.beast_count {
        selected_roles.push(werewolf.id);
    }
    if roles.citizens.is_empty() {
        return Err("No citizens available".to_string());
    }
    let citizens = roles.citizens.first().unwrap();
    for _ in 0..distribution.citizen_count {
        selected_roles.push(citizens.id);
    }

    let special_to_assign = distribution.special_count.min(roles.special.len());
    let mut specials: Vec<_> = roles.special
        .choose_multiple(&mut rng, special_to_assign)
        .collect();
    for role in specials {
        selected_roles.push(role.id);
    }

    selected_roles.shuffle(&mut rng);
    Ok(selected_roles)
}