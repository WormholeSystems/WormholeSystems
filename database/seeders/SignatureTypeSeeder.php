<?php

declare(strict_types=1);

namespace Database\Seeders;

use App\Enums\WormholeClass;
use App\Enums\WormholeSignature;
use App\Enums\WormholeTargetClass;
use App\Models\SignatureCategory;
use App\Models\SignatureType;
use Illuminate\Database\Seeder;

final class SignatureTypeSeeder extends Seeder
{
    public function run(): void
    {
        $this->seedCombatSites();
        $this->seedRelicSites();
        $this->seedDataSites();
        $this->seedGasSites();
        $this->seedOreSites();
        $this->seedWormholes();
    }

    private function seedCombatSites(): void
    {
        $sites = [
            // C1
            ['name' => 'Perimeter Ambush Point', 'spawn_areas' => [WormholeClass::C1]],
            ['name' => 'Perimeter Camp', 'spawn_areas' => [WormholeClass::C1]],
            ['name' => 'Phase Catalyst Node', 'spawn_areas' => [WormholeClass::C1]],
            ['name' => 'The Line', 'spawn_areas' => [WormholeClass::C1]],
            // C2
            ['name' => 'Perimeter Checkpoint', 'spawn_areas' => [WormholeClass::C2]],
            ['name' => 'Perimeter Hangar', 'spawn_areas' => [WormholeClass::C2]],
            ['name' => 'The Ruins of Enclave Cohort 27', 'spawn_areas' => [WormholeClass::C2]],
            ['name' => 'Sleeper Data Sanctuary', 'spawn_areas' => [WormholeClass::C2]],
            // C3
            ['name' => 'Fortification Frontier Stronghold', 'spawn_areas' => [WormholeClass::C3]],
            ['name' => 'Outpost Frontier Stronghold', 'spawn_areas' => [WormholeClass::C3]],
            ['name' => 'Solar Cell', 'spawn_areas' => [WormholeClass::C3]],
            ['name' => 'The Oruze Construct', 'spawn_areas' => [WormholeClass::C3]],
            // C4
            ['name' => 'Frontier Barracks', 'spawn_areas' => [WormholeClass::C4]],
            ['name' => 'Frontier Command Post', 'spawn_areas' => [WormholeClass::C4]],
            ['name' => 'Integrated Terminus', 'spawn_areas' => [WormholeClass::C4]],
            ['name' => 'Sleeper Information Sanctum', 'spawn_areas' => [WormholeClass::C4]],
            // C5
            ['name' => 'Core Garrison', 'spawn_areas' => [WormholeClass::C5]],
            ['name' => 'Core Stronghold', 'spawn_areas' => [WormholeClass::C5]],
            ['name' => 'Oruze Osobnyk', 'spawn_areas' => [WormholeClass::C5]],
            ['name' => 'Quarantine Area', 'spawn_areas' => [WormholeClass::C5]],
            // C6
            ['name' => 'Core Citadel', 'spawn_areas' => [WormholeClass::C6]],
            ['name' => 'Core Bastion', 'spawn_areas' => [WormholeClass::C6]],
            ['name' => 'Strange Energy Readings', 'spawn_areas' => [WormholeClass::C6]],
            ['name' => 'The Mirror', 'spawn_areas' => [WormholeClass::C6]],
            // C12 (Thera)
            ['name' => 'Epicenter', 'spawn_areas' => [WormholeClass::C12]],
            ['name' => 'Expedition Command Outpost Wreck', 'spawn_areas' => [WormholeClass::C12]],
            ['name' => 'Planetary Colonization Office Wreck', 'spawn_areas' => [WormholeClass::C12]],
            ['name' => 'Testing Facilities', 'spawn_areas' => [WormholeClass::C12]],
            // C14 (Drifter Sentinel)
            ['name' => 'Monolith', 'spawn_areas' => [WormholeClass::C14]],
            ['name' => 'Wormhole in Rock Circle', 'spawn_areas' => [WormholeClass::C14]],
            ['name' => 'Opposing Spatial Rifts', 'spawn_areas' => [WormholeClass::C14]],
            ['name' => 'Sleeper Enclave Debris', 'spawn_areas' => [WormholeClass::C14]],
            ['name' => 'Crystal Resource', 'spawn_areas' => [WormholeClass::C14]],
            // C15 (Drifter Barbican)
            ['name' => 'Wrecked Ships', 'spawn_areas' => [WormholeClass::C15]],
            ['name' => 'Unstable Wormhole', 'spawn_areas' => [WormholeClass::C15]],
            ['name' => 'Spatial Rift', 'spawn_areas' => [WormholeClass::C15, WormholeClass::C16]],
            ['name' => 'Heavily Guarded Spatial Rift', 'spawn_areas' => [WormholeClass::C15]],
            ['name' => 'Crystals', 'spawn_areas' => [WormholeClass::C15]],
            // C16 (Drifter Vidette)
            ['name' => 'Ship Graveyard', 'spawn_areas' => [WormholeClass::C16, WormholeClass::C18]],
            ['name' => 'Sleeper Engineering Station', 'spawn_areas' => [WormholeClass::C16]],
            ['name' => 'Sleeper Enclave in Coral Rock', 'spawn_areas' => [WormholeClass::C16]],
            ['name' => 'Crystals and Stone Circle', 'spawn_areas' => [WormholeClass::C16]],
            // C17 (Drifter Conflux)
            ['name' => 'Caged Wormhole', 'spawn_areas' => [WormholeClass::C17, WormholeClass::C18]],
            ['name' => 'Rock Formation and Wormhole', 'spawn_areas' => [WormholeClass::C17]],
            ['name' => 'Particle Acceleration Array', 'spawn_areas' => [WormholeClass::C17]],
            ['name' => 'Guarded Asteroid Station', 'spawn_areas' => [WormholeClass::C17]],
            // C18 (Drifter Redoubt)
            ['name' => 'Spatial Rift Generator', 'spawn_areas' => [WormholeClass::C18]],
            ['name' => 'Sleeper Enclave', 'spawn_areas' => [WormholeClass::C18]],
            ['name' => 'Hollow Asteroid', 'spawn_areas' => [WormholeClass::C18]],
        ];

        $category = SignatureCategory::where('code', 'combat')->firstOrFail();
        foreach ($sites as $site) {
            SignatureType::updateOrCreate(
                ['name' => $site['name'], 'signature_category_id' => $category->id],
                array_merge($site, ['signature_category_id' => $category->id])
            );
        }
    }

    private function seedRelicSites(): void
    {
        $sites = [
            // Null sec relic sites (appear in C1-C3)
            ['name' => 'Ruined Angel Crystal Quarry', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Ruined Angel Monument Site', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Ruined Angel Science Outpost', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Ruined Angel Temple Site', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Ruined Blood Raider Crystal Quarry', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Ruined Blood Raider Monument Site', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Ruined Blood Raider Science Outpost', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Ruined Blood Raider Temple Site', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Ruined Guristas Crystal Quarry', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Ruined Guristas Monument Site', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Ruined Guristas Science Outpost', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Ruined Guristas Temple Site', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Ruined Sansha Crystal Quarry', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Ruined Sansha Monument Site', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Ruined Sansha Science Outpost', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Ruined Sansha Temple Site', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Ruined Serpentis Crystal Quarry', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Ruined Serpentis Monument Site', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Ruined Serpentis Science Outpost', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Ruined Serpentis Temple Site', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            // Class-specific relic sites
            ['name' => 'Forgotten Perimeter Coronation Platform', 'spawn_areas' => [WormholeClass::C1]],
            ['name' => 'Forgotten Perimeter Power Array', 'spawn_areas' => [WormholeClass::C1]],
            ['name' => 'Forgotten Perimeter Gateway', 'spawn_areas' => [WormholeClass::C2]],
            ['name' => 'Forgotten Perimeter Habitation Coils', 'spawn_areas' => [WormholeClass::C2]],
            ['name' => 'Forgotten Frontier Quarantine Outpost', 'spawn_areas' => [WormholeClass::C3]],
            ['name' => 'Forgotten Frontier Recursive Depot', 'spawn_areas' => [WormholeClass::C3]],
            ['name' => 'Forgotten Frontier Conversion Module', 'spawn_areas' => [WormholeClass::C4]],
            ['name' => 'Forgotten Frontier Evacuation Center', 'spawn_areas' => [WormholeClass::C4]],
            ['name' => 'Forgotten Core Data Field', 'spawn_areas' => [WormholeClass::C5]],
            ['name' => 'Forgotten Core Information Pen', 'spawn_areas' => [WormholeClass::C5]],
            ['name' => 'Forgotten Core Assembly Hall', 'spawn_areas' => [WormholeClass::C6]],
            ['name' => 'Forgotten Core Circuitry Disassembler', 'spawn_areas' => [WormholeClass::C6]],
        ];

        $category = SignatureCategory::where('code', 'relic')->firstOrFail();
        foreach ($sites as $site) {
            SignatureType::updateOrCreate(
                ['name' => $site['name'], 'signature_category_id' => $category->id],
                array_merge($site, ['signature_category_id' => $category->id])
            );
        }
    }

    private function seedDataSites(): void
    {
        $sites = [
            // Null sec data sites (appear in C1-C3)
            ['name' => 'Abandoned Research Complex DA005', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Abandoned Research Complex DA015', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Abandoned Research Complex DC007', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Abandoned Research Complex DC021', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Abandoned Research Complex DC035', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Abandoned Research Complex DG003', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Central Angel Command Center', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Central Angel Data Mining Site', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Central Angel Sparking Transmitter', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Central Angel Survey Site', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Central Blood Raider Command Center', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Central Blood Raider Data Mining Site', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Central Blood Raider Sparking Transmitter', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Central Blood Raider Survey Site', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Central Guristas Command Center', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Central Guristas Data Mining Site', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Central Guristas Sparking Transmitter', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Central Guristas Survey Site', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Central Sansha Command Center', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Central Sansha Data Mining Site', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Central Sansha Sparking Transmitter', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Central Sansha Survey Site', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Central Serpentis Command Center', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Central Serpentis Data Mining Site', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Central Serpentis Sparking Transmitter', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            ['name' => 'Central Serpentis Survey Site', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3]],
            // Ghost sites
            ['name' => 'Lesser Serpentis Covert Research Facility', 'spawn_areas' => [WormholeClass::HighSec]],
            ['name' => 'Lesser Sansha Covert Research Facility', 'spawn_areas' => [WormholeClass::HighSec]],
            ['name' => 'Lesser Guristas Covert Research Facility', 'spawn_areas' => [WormholeClass::HighSec]],
            ['name' => 'Lesser Blood Raiders Covert Research Facility', 'spawn_areas' => [WormholeClass::HighSec]],
            ['name' => 'Standard Serpentis Covert Research Facility', 'spawn_areas' => [WormholeClass::LowSec]],
            ['name' => 'Standard Sansha Covert Research Facility', 'spawn_areas' => [WormholeClass::LowSec]],
            ['name' => 'Standard Guristas Covert Research Facility', 'spawn_areas' => [WormholeClass::LowSec]],
            ['name' => 'Standard Blood Raiders Covert Research Facility', 'spawn_areas' => [WormholeClass::LowSec]],
            ['name' => 'Improved Serpentis Covert Research Facility', 'spawn_areas' => [WormholeClass::NullSec]],
            ['name' => 'Improved Sansha Covert Research Facility', 'spawn_areas' => [WormholeClass::NullSec]],
            ['name' => 'Improved Guristas Covert Research Facility', 'spawn_areas' => [WormholeClass::NullSec]],
            ['name' => 'Improved Blood Raiders Covert Research Facility', 'spawn_areas' => [WormholeClass::NullSec]],
            ['name' => 'Superior Serpentis Covert Research Facility', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6, WormholeClass::C13]],
            ['name' => 'Superior Sansha Covert Research Facility', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6, WormholeClass::C13]],
            ['name' => 'Superior Guristas Covert Research Facility', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6, WormholeClass::C13]],
            ['name' => 'Superior Blood Raiders Covert Research Facility', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6, WormholeClass::C13]],
            // Class-specific data sites
            ['name' => 'Unsecured Perimeter Amplifier', 'spawn_areas' => [WormholeClass::C1]],
            ['name' => 'Unsecured Perimeter Information Center', 'spawn_areas' => [WormholeClass::C1]],
            ['name' => 'Unsecured Perimeter Comms Relay', 'spawn_areas' => [WormholeClass::C2]],
            ['name' => 'Unsecured Perimeter Transponder Farm', 'spawn_areas' => [WormholeClass::C2]],
            ['name' => 'Unsecured Frontier Database', 'spawn_areas' => [WormholeClass::C3]],
            ['name' => 'Unsecured Frontier Receiver', 'spawn_areas' => [WormholeClass::C3]],
            ['name' => 'Unsecured Frontier Digital Nexus', 'spawn_areas' => [WormholeClass::C4]],
            ['name' => 'Unsecured Frontier Trinary Hub', 'spawn_areas' => [WormholeClass::C4]],
            ['name' => 'Unsecured Frontier Enclave Relay', 'spawn_areas' => [WormholeClass::C5]],
            ['name' => 'Unsecured Frontier Server Bank', 'spawn_areas' => [WormholeClass::C5]],
            ['name' => 'Unsecured Core Backup Array', 'spawn_areas' => [WormholeClass::C6]],
            ['name' => 'Unsecured Core Emergence', 'spawn_areas' => [WormholeClass::C6]],
        ];

        $category = SignatureCategory::where('code', 'data')->firstOrFail();
        foreach ($sites as $site) {
            SignatureType::updateOrCreate(
                ['name' => $site['name'], 'signature_category_id' => $category->id],
                array_merge($site, ['signature_category_id' => $category->id])
            );
        }
    }

    private function seedGasSites(): void
    {
        $sites = [
            ['name' => 'Barren Perimeter Reservoir', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6]],
            ['name' => 'Token Perimeter Reservoir', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6]],
            ['name' => 'Minor Perimeter Reservoir', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6]],
            ['name' => 'Sizeable Perimeter Reservoir', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6]],
            ['name' => 'Ordinary Perimeter Reservoir', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6]],
            ['name' => 'Bountiful Frontier Reservoir', 'spawn_areas' => [WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6]],
            ['name' => 'Vast Frontier Reservoir', 'spawn_areas' => [WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6]],
            ['name' => 'Instrumental Core Reservoir', 'spawn_areas' => [WormholeClass::C5, WormholeClass::C6]],
            ['name' => 'Vital Core Reservoir', 'spawn_areas' => [WormholeClass::C5, WormholeClass::C6]],
        ];

        $category = SignatureCategory::where('code', 'gas')->firstOrFail();
        foreach ($sites as $site) {
            SignatureType::updateOrCreate(
                ['name' => $site['name'], 'signature_category_id' => $category->id],
                array_merge($site, ['signature_category_id' => $category->id])
            );
        }
    }

    private function seedOreSites(): void
    {
        $sites = [
            ['name' => 'Ordinary Perimeter Deposit', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6]],
            ['name' => 'Common Perimeter Deposit', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6]],
            ['name' => 'Unexceptional Frontier Deposit', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6]],
            ['name' => 'Average Frontier Deposit', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5]],
            ['name' => 'Isolated Core Deposit', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C5]],
            ['name' => 'Uncommon Core Deposit', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C5]],
            ['name' => 'Infrequent Core Deposit', 'spawn_areas' => [WormholeClass::C3, WormholeClass::C4, WormholeClass::C5]],
            ['name' => 'Unusual Core Deposit', 'spawn_areas' => [WormholeClass::C3, WormholeClass::C4, WormholeClass::C5]],
            ['name' => 'Exceptional Core Deposit', 'spawn_areas' => [WormholeClass::C5]],
            ['name' => 'Rarified Core Deposit', 'spawn_areas' => [WormholeClass::C5, WormholeClass::C6]],
            ['name' => 'Shattered Debris Field', 'spawn_areas' => [WormholeClass::C13]],
            ['name' => 'Shattered Ice Field', 'spawn_areas' => [WormholeClass::C13]],
        ];

        $category = SignatureCategory::where('code', 'ore')->firstOrFail();
        foreach ($sites as $site) {
            SignatureType::updateOrCreate(
                ['name' => $site['name'], 'signature_category_id' => $category->id],
                array_merge($site, ['signature_category_id' => $category->id])
            );
        }
    }

    private function seedWormholes(): void
    {
        // Due to the large size, wormholes are seeded in batches
        $this->seedK162Wormholes();
        $this->seedUniversalWormholes();
        $this->seedC1Wormholes();
        $this->seedC2Wormholes();
        $this->seedC3Wormholes();
        $this->seedC4Wormholes();
        $this->seedC5Wormholes();
        $this->seedC6Wormholes();
        $this->seedKSpaceWormholes();
        $this->seedSpecialWormholes();
    }

    private function seedK162Wormholes(): void
    {
        $allClasses = [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6, WormholeClass::C13, WormholeClass::C14, WormholeClass::C15, WormholeClass::C16, WormholeClass::C17, WormholeClass::C18, WormholeClass::HighSec, WormholeClass::LowSec, WormholeClass::NullSec, WormholeClass::Pochven];
        $wormholes = [
            ['name' => 'K162 - Unknown', 'signature' => WormholeSignature::K162, 'target_class' => WormholeTargetClass::Unknown, 'spawn_areas' => $allClasses],
            ['name' => 'K162 - C1', 'signature' => WormholeSignature::K162, 'target_class' => WormholeTargetClass::C1, 'spawn_areas' => $allClasses],
            ['name' => 'K162 - C2', 'signature' => WormholeSignature::K162, 'target_class' => WormholeTargetClass::C2, 'spawn_areas' => $allClasses],
            ['name' => 'K162 - C3', 'signature' => WormholeSignature::K162, 'target_class' => WormholeTargetClass::C3, 'spawn_areas' => $allClasses],
            ['name' => 'K162 - C4', 'signature' => WormholeSignature::K162, 'target_class' => WormholeTargetClass::C4, 'spawn_areas' => $allClasses],
            ['name' => 'K162 - C5', 'signature' => WormholeSignature::K162, 'target_class' => WormholeTargetClass::C5, 'spawn_areas' => $allClasses],
            ['name' => 'K162 - C6', 'signature' => WormholeSignature::K162, 'target_class' => WormholeTargetClass::C6, 'spawn_areas' => $allClasses],
            ['name' => 'K162 - H', 'signature' => WormholeSignature::K162, 'target_class' => WormholeTargetClass::H, 'spawn_areas' => $allClasses],
            ['name' => 'K162 - L', 'signature' => WormholeSignature::K162, 'target_class' => WormholeTargetClass::L, 'spawn_areas' => $allClasses],
            ['name' => 'K162 - N', 'signature' => WormholeSignature::K162, 'target_class' => WormholeTargetClass::N, 'spawn_areas' => $allClasses],
        ];

        $category = SignatureCategory::where('code', 'wormhole')->firstOrFail();
        foreach ($wormholes as $wormhole) {
            SignatureType::updateOrCreate(
                ['name' => $wormhole['name'], 'signature_category_id' => $category->id],
                array_merge($wormhole, ['signature_category_id' => $category->id])
            );
        }
    }

    private function seedUniversalWormholes(): void
    {
        $wormholes = [
            ['name' => 'A009 - C13', 'signature' => WormholeSignature::A009, 'target_class' => WormholeTargetClass::C13, 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6, WormholeClass::C13, WormholeClass::HighSec, WormholeClass::LowSec, WormholeClass::NullSec, WormholeClass::Pochven]],
            ['name' => 'B735 - Barbican (C15)', 'signature' => WormholeSignature::B735, 'target_class' => WormholeTargetClass::C15, 'extra' => 'Barbican', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6, WormholeClass::HighSec, WormholeClass::LowSec, WormholeClass::NullSec, WormholeClass::Pochven]],
            ['name' => 'C008 - C5', 'signature' => WormholeSignature::C008, 'target_class' => WormholeTargetClass::C5, 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6, WormholeClass::HighSec, WormholeClass::LowSec, WormholeClass::NullSec, WormholeClass::Pochven]],
            ['name' => 'C414 - Conflux (C17)', 'signature' => WormholeSignature::C414, 'target_class' => WormholeTargetClass::C17, 'extra' => 'Conflux', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6, WormholeClass::HighSec, WormholeClass::LowSec, WormholeClass::NullSec, WormholeClass::Pochven]],
            ['name' => 'E004 - C1', 'signature' => WormholeSignature::E004, 'target_class' => WormholeTargetClass::C1, 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6, WormholeClass::C13, WormholeClass::HighSec, WormholeClass::LowSec, WormholeClass::NullSec, WormholeClass::Pochven]],
            ['name' => 'G008 - C6', 'signature' => WormholeSignature::G008, 'target_class' => WormholeTargetClass::C6, 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6, WormholeClass::HighSec, WormholeClass::LowSec, WormholeClass::NullSec, WormholeClass::Pochven]],
            ['name' => 'J492 - L', 'signature' => WormholeSignature::J492, 'target_class' => WormholeTargetClass::L, 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6]],
            ['name' => 'L005 - C2', 'signature' => WormholeSignature::L005, 'target_class' => WormholeTargetClass::C2, 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6, WormholeClass::C13, WormholeClass::HighSec, WormholeClass::LowSec, WormholeClass::NullSec, WormholeClass::Pochven]],
            ['name' => 'M001 - C4', 'signature' => WormholeSignature::M001, 'target_class' => WormholeTargetClass::C4, 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6, WormholeClass::C13, WormholeClass::HighSec, WormholeClass::LowSec, WormholeClass::NullSec, WormholeClass::Pochven]],
            ['name' => 'Q003 - 0.0', 'signature' => WormholeSignature::Q003, 'target_class' => WormholeTargetClass::N, 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6, WormholeClass::C13, WormholeClass::HighSec, WormholeClass::LowSec, WormholeClass::NullSec, WormholeClass::Pochven]],
            ['name' => 'R259 - Redoubt (C18)', 'signature' => WormholeSignature::R259, 'target_class' => WormholeTargetClass::C18, 'extra' => 'Redoubt', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6, WormholeClass::HighSec, WormholeClass::LowSec, WormholeClass::NullSec, WormholeClass::Pochven]],
            ['name' => 'S877 - Sentinel (C14)', 'signature' => WormholeSignature::S877, 'target_class' => WormholeTargetClass::C14, 'extra' => 'Sentinel', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6, WormholeClass::HighSec, WormholeClass::LowSec, WormholeClass::NullSec, WormholeClass::Pochven]],
            ['name' => 'V928 - Vidette (C16)', 'signature' => WormholeSignature::V928, 'target_class' => WormholeTargetClass::C16, 'extra' => 'Vidette', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6, WormholeClass::HighSec, WormholeClass::LowSec, WormholeClass::NullSec, WormholeClass::Pochven]],
            ['name' => 'Z006 - C3', 'signature' => WormholeSignature::Z006, 'target_class' => WormholeTargetClass::C3, 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6, WormholeClass::C13, WormholeClass::HighSec, WormholeClass::LowSec, WormholeClass::NullSec, WormholeClass::Pochven]],
        ];

        $category = SignatureCategory::where('code', 'wormhole')->firstOrFail();
        foreach ($wormholes as $wormhole) {
            SignatureType::updateOrCreate(
                ['name' => $wormhole['name'], 'signature_category_id' => $category->id],
                array_merge($wormhole, ['signature_category_id' => $category->id])
            );
        }
    }

    private function seedC1Wormholes(): void
    {
        $wormholes = [
            ['name' => 'H121 - C1', 'signature' => WormholeSignature::H121, 'target_class' => WormholeTargetClass::C1, 'spawn_areas' => [WormholeClass::C1]],
            ['name' => 'C125 - C2', 'signature' => WormholeSignature::C125, 'target_class' => WormholeTargetClass::C2, 'spawn_areas' => [WormholeClass::C1]],
            ['name' => 'O883 - C3', 'signature' => WormholeSignature::O883, 'target_class' => WormholeTargetClass::C3, 'spawn_areas' => [WormholeClass::C1]],
            ['name' => 'M609 - C4', 'signature' => WormholeSignature::M609, 'target_class' => WormholeTargetClass::C4, 'spawn_areas' => [WormholeClass::C1]],
            ['name' => 'L614 - C5', 'signature' => WormholeSignature::L614, 'target_class' => WormholeTargetClass::C5, 'spawn_areas' => [WormholeClass::C1]],
            ['name' => 'S804 - C6', 'signature' => WormholeSignature::S804, 'target_class' => WormholeTargetClass::C6, 'spawn_areas' => [WormholeClass::C1]],
            ['name' => 'N110 - H', 'signature' => WormholeSignature::N110, 'target_class' => WormholeTargetClass::H, 'spawn_areas' => [WormholeClass::C1, WormholeClass::C13]],
            ['name' => 'J244 - L', 'signature' => WormholeSignature::J244, 'target_class' => WormholeTargetClass::L, 'spawn_areas' => [WormholeClass::C1, WormholeClass::C13]],
            ['name' => 'Z060 - 0.0', 'signature' => WormholeSignature::Z060, 'target_class' => WormholeTargetClass::N, 'spawn_areas' => [WormholeClass::C1, WormholeClass::C13]],
            ['name' => 'F353 - C12 Thera', 'signature' => WormholeSignature::F353, 'target_class' => WormholeTargetClass::C12, 'extra' => 'Thera', 'spawn_areas' => [WormholeClass::C1]],
        ];

        $category = SignatureCategory::where('code', 'wormhole')->firstOrFail();
        foreach ($wormholes as $wormhole) {
            SignatureType::updateOrCreate(
                ['name' => $wormhole['name'], 'signature_category_id' => $category->id],
                array_merge($wormhole, ['signature_category_id' => $category->id])
            );
        }
    }

    private function seedC2Wormholes(): void
    {
        $wormholes = [
            ['name' => 'Z647 - C1', 'signature' => WormholeSignature::Z647, 'target_class' => WormholeTargetClass::C1, 'spawn_areas' => [WormholeClass::C2, WormholeClass::C13, WormholeClass::C14, WormholeClass::C15, WormholeClass::C16, WormholeClass::C17, WormholeClass::C18]],
            ['name' => 'D382 - C2', 'signature' => WormholeSignature::D382, 'target_class' => WormholeTargetClass::C2, 'spawn_areas' => [WormholeClass::C2, WormholeClass::C13, WormholeClass::C14, WormholeClass::C15, WormholeClass::C16, WormholeClass::C17, WormholeClass::C18]],
            ['name' => 'O477 - C3', 'signature' => WormholeSignature::O477, 'target_class' => WormholeTargetClass::C3, 'spawn_areas' => [WormholeClass::C2, WormholeClass::C13, WormholeClass::C14, WormholeClass::C15, WormholeClass::C16, WormholeClass::C17, WormholeClass::C18]],
            ['name' => 'Y683 - C4', 'signature' => WormholeSignature::Y683, 'target_class' => WormholeTargetClass::C4, 'spawn_areas' => [WormholeClass::C2, WormholeClass::C13, WormholeClass::C14, WormholeClass::C15, WormholeClass::C16, WormholeClass::C17, WormholeClass::C18]],
            ['name' => 'N062 - C5', 'signature' => WormholeSignature::N062, 'target_class' => WormholeTargetClass::C5, 'spawn_areas' => [WormholeClass::C2, WormholeClass::C13, WormholeClass::C14, WormholeClass::C15, WormholeClass::C16, WormholeClass::C17, WormholeClass::C18]],
            ['name' => 'R474 - C6', 'signature' => WormholeSignature::R474, 'target_class' => WormholeTargetClass::C6, 'spawn_areas' => [WormholeClass::C2, WormholeClass::C14, WormholeClass::C15, WormholeClass::C16, WormholeClass::C17, WormholeClass::C18]],
            ['name' => 'B274 - H', 'signature' => WormholeSignature::B274, 'target_class' => WormholeTargetClass::H, 'spawn_areas' => [WormholeClass::C2, WormholeClass::C13]],
            ['name' => 'A239 - L', 'signature' => WormholeSignature::A239, 'target_class' => WormholeTargetClass::L, 'spawn_areas' => [WormholeClass::C2, WormholeClass::C13]],
            ['name' => 'E545 - 0.0', 'signature' => WormholeSignature::E545, 'target_class' => WormholeTargetClass::N, 'spawn_areas' => [WormholeClass::C2, WormholeClass::C13]],
            ['name' => 'F135 - C12 Thera', 'signature' => WormholeSignature::F135, 'target_class' => WormholeTargetClass::C12, 'extra' => 'Thera', 'spawn_areas' => [WormholeClass::C2, WormholeClass::C3]],
        ];

        $category = SignatureCategory::where('code', 'wormhole')->firstOrFail();
        foreach ($wormholes as $wormhole) {
            SignatureType::updateOrCreate(
                ['name' => $wormhole['name'], 'signature_category_id' => $category->id],
                array_merge($wormhole, ['signature_category_id' => $category->id])
            );
        }
    }

    private function seedC3Wormholes(): void
    {
        $wormholes = [
            ['name' => 'V301 - C1', 'signature' => WormholeSignature::V301, 'target_class' => WormholeTargetClass::C1, 'spawn_areas' => [WormholeClass::C3]],
            ['name' => 'I182 - C2', 'signature' => WormholeSignature::I182, 'target_class' => WormholeTargetClass::C2, 'spawn_areas' => [WormholeClass::C3]],
            ['name' => 'N968 - C3', 'signature' => WormholeSignature::N968, 'target_class' => WormholeTargetClass::C3, 'spawn_areas' => [WormholeClass::C3]],
            ['name' => 'T405 - C4', 'signature' => WormholeSignature::T405, 'target_class' => WormholeTargetClass::C4, 'spawn_areas' => [WormholeClass::C3]],
            ['name' => 'N770 - C5', 'signature' => WormholeSignature::N770, 'target_class' => WormholeTargetClass::C5, 'spawn_areas' => [WormholeClass::C3]],
            ['name' => 'A982 - C6', 'signature' => WormholeSignature::A982, 'target_class' => WormholeTargetClass::C6, 'spawn_areas' => [WormholeClass::C3]],
            ['name' => 'D845 - H', 'signature' => WormholeSignature::D845, 'target_class' => WormholeTargetClass::H, 'spawn_areas' => [WormholeClass::C3, WormholeClass::C13]],
            ['name' => 'U210 - L', 'signature' => WormholeSignature::U210, 'target_class' => WormholeTargetClass::L, 'spawn_areas' => [WormholeClass::C3, WormholeClass::C13]],
            ['name' => 'K346 - 0.0', 'signature' => WormholeSignature::K346, 'target_class' => WormholeTargetClass::N, 'spawn_areas' => [WormholeClass::C3, WormholeClass::C13]],
        ];

        $category = SignatureCategory::where('code', 'wormhole')->firstOrFail();
        foreach ($wormholes as $wormhole) {
            SignatureType::updateOrCreate(
                ['name' => $wormhole['name'], 'signature_category_id' => $category->id],
                array_merge($wormhole, ['signature_category_id' => $category->id])
            );
        }
    }

    private function seedC4Wormholes(): void
    {
        $wormholes = [
            ['name' => 'P060 - C1', 'signature' => WormholeSignature::P060, 'target_class' => WormholeTargetClass::C1, 'spawn_areas' => [WormholeClass::C4, WormholeClass::C13]],
            ['name' => 'N766 - C2', 'signature' => WormholeSignature::N766, 'target_class' => WormholeTargetClass::C2, 'spawn_areas' => [WormholeClass::C4, WormholeClass::C13]],
            ['name' => 'C247 - C3', 'signature' => WormholeSignature::C247, 'target_class' => WormholeTargetClass::C3, 'spawn_areas' => [WormholeClass::C4, WormholeClass::C13]],
            ['name' => 'X877 - C4', 'signature' => WormholeSignature::X877, 'target_class' => WormholeTargetClass::C4, 'spawn_areas' => [WormholeClass::C4, WormholeClass::C13]],
            ['name' => 'H900 - C5', 'signature' => WormholeSignature::H900, 'target_class' => WormholeTargetClass::C5, 'spawn_areas' => [WormholeClass::C4, WormholeClass::C13]],
            ['name' => 'U574 - C6', 'signature' => WormholeSignature::U574, 'target_class' => WormholeTargetClass::C6, 'spawn_areas' => [WormholeClass::C4, WormholeClass::C13]],
            ['name' => 'S047 - H', 'signature' => WormholeSignature::S047, 'target_class' => WormholeTargetClass::H, 'spawn_areas' => [WormholeClass::C4]],
            ['name' => 'N290 - L', 'signature' => WormholeSignature::N290, 'target_class' => WormholeTargetClass::L, 'spawn_areas' => [WormholeClass::C4]],
            ['name' => 'K329 - 0.0', 'signature' => WormholeSignature::K329, 'target_class' => WormholeTargetClass::N, 'spawn_areas' => [WormholeClass::C4]],
        ];

        $category = SignatureCategory::where('code', 'wormhole')->firstOrFail();
        foreach ($wormholes as $wormhole) {
            SignatureType::updateOrCreate(
                ['name' => $wormhole['name'], 'signature_category_id' => $category->id],
                array_merge($wormhole, ['signature_category_id' => $category->id])
            );
        }
    }

    private function seedC5Wormholes(): void
    {
        $wormholes = [
            ['name' => 'Y790 - C1', 'signature' => WormholeSignature::Y790, 'target_class' => WormholeTargetClass::C1, 'spawn_areas' => [WormholeClass::C5]],
            ['name' => 'D364 - C2', 'signature' => WormholeSignature::D364, 'target_class' => WormholeTargetClass::C2, 'spawn_areas' => [WormholeClass::C5]],
            ['name' => 'M267 - C3', 'signature' => WormholeSignature::M267, 'target_class' => WormholeTargetClass::C3, 'spawn_areas' => [WormholeClass::C5, WormholeClass::C13]],
            ['name' => 'E175 - C4', 'signature' => WormholeSignature::E175, 'target_class' => WormholeTargetClass::C4, 'spawn_areas' => [WormholeClass::C5]],
            ['name' => 'H296 - C5', 'signature' => WormholeSignature::H296, 'target_class' => WormholeTargetClass::C5, 'spawn_areas' => [WormholeClass::C5, WormholeClass::C13]],
            ['name' => 'V753 - C6', 'signature' => WormholeSignature::V753, 'target_class' => WormholeTargetClass::C6, 'spawn_areas' => [WormholeClass::C5, WormholeClass::C13]],
            ['name' => 'D792 - H', 'signature' => WormholeSignature::D792, 'target_class' => WormholeTargetClass::H, 'spawn_areas' => [WormholeClass::C5, WormholeClass::C6, WormholeClass::C13]],
            ['name' => 'C140 - L', 'signature' => WormholeSignature::C140, 'target_class' => WormholeTargetClass::L, 'spawn_areas' => [WormholeClass::C5, WormholeClass::C6]],
            ['name' => 'Z142 - 0.0', 'signature' => WormholeSignature::Z142, 'target_class' => WormholeTargetClass::N, 'spawn_areas' => [WormholeClass::C5, WormholeClass::C6]],
        ];

        $category = SignatureCategory::where('code', 'wormhole')->firstOrFail();
        foreach ($wormholes as $wormhole) {
            SignatureType::updateOrCreate(
                ['name' => $wormhole['name'], 'signature_category_id' => $category->id],
                array_merge($wormhole, ['signature_category_id' => $category->id])
            );
        }
    }

    private function seedC6Wormholes(): void
    {
        $wormholes = [
            ['name' => 'Q317 - C1', 'signature' => WormholeSignature::Q317, 'target_class' => WormholeTargetClass::C1, 'spawn_areas' => [WormholeClass::C6, WormholeClass::C13]],
            ['name' => 'G024 - C2', 'signature' => WormholeSignature::G024, 'target_class' => WormholeTargetClass::C2, 'spawn_areas' => [WormholeClass::C6, WormholeClass::C13]],
            ['name' => 'L477 - C3', 'signature' => WormholeSignature::L477, 'target_class' => WormholeTargetClass::C3, 'spawn_areas' => [WormholeClass::C6]],
            ['name' => 'Z457 - C4', 'signature' => WormholeSignature::Z457, 'target_class' => WormholeTargetClass::C4, 'spawn_areas' => [WormholeClass::C6]],
            ['name' => 'V911 - C5', 'signature' => WormholeSignature::V911, 'target_class' => WormholeTargetClass::C5, 'spawn_areas' => [WormholeClass::C6, WormholeClass::C13]],
            ['name' => 'W237 - C6', 'signature' => WormholeSignature::W237, 'target_class' => WormholeTargetClass::C6, 'spawn_areas' => [WormholeClass::C6, WormholeClass::C13]],
            ['name' => 'B520 - H', 'signature' => WormholeSignature::B520, 'target_class' => WormholeTargetClass::H, 'spawn_areas' => [WormholeClass::C6]],
            ['name' => 'C391 - L', 'signature' => WormholeSignature::C391, 'target_class' => WormholeTargetClass::L, 'spawn_areas' => [WormholeClass::C6, WormholeClass::C13]],
            ['name' => 'C248 - 0.0', 'signature' => WormholeSignature::C248, 'target_class' => WormholeTargetClass::N, 'spawn_areas' => [WormholeClass::C6, WormholeClass::C13]],
        ];

        $category = SignatureCategory::where('code', 'wormhole')->firstOrFail();
        foreach ($wormholes as $wormhole) {
            SignatureType::updateOrCreate(
                ['name' => $wormhole['name'], 'signature_category_id' => $category->id],
                array_merge($wormhole, ['signature_category_id' => $category->id])
            );
        }
    }

    private function seedKSpaceWormholes(): void
    {
        $wormholes = [
            // Highsec
            ['name' => 'Z971 - C1', 'signature' => WormholeSignature::Z971, 'target_class' => WormholeTargetClass::C1, 'spawn_areas' => [WormholeClass::HighSec, WormholeClass::LowSec, WormholeClass::NullSec]],
            ['name' => 'R943 - C2', 'signature' => WormholeSignature::R943, 'target_class' => WormholeTargetClass::C2, 'spawn_areas' => [WormholeClass::HighSec, WormholeClass::LowSec, WormholeClass::NullSec]],
            ['name' => 'X702 - C3', 'signature' => WormholeSignature::X702, 'target_class' => WormholeTargetClass::C3, 'spawn_areas' => [WormholeClass::HighSec, WormholeClass::LowSec, WormholeClass::NullSec]],
            ['name' => 'O128 - C4', 'signature' => WormholeSignature::O128, 'target_class' => WormholeTargetClass::C4, 'spawn_areas' => [WormholeClass::HighSec, WormholeClass::LowSec, WormholeClass::NullSec]],
            ['name' => 'M555 - C5', 'signature' => WormholeSignature::M555, 'target_class' => WormholeTargetClass::C5, 'spawn_areas' => [WormholeClass::HighSec]],
            ['name' => 'B041 - C6', 'signature' => WormholeSignature::B041, 'target_class' => WormholeTargetClass::C6, 'spawn_areas' => [WormholeClass::HighSec]],
            ['name' => 'A641 - H', 'signature' => WormholeSignature::A641, 'target_class' => WormholeTargetClass::H, 'spawn_areas' => [WormholeClass::HighSec]],
            ['name' => 'R051 - L', 'signature' => WormholeSignature::R051, 'target_class' => WormholeTargetClass::L, 'spawn_areas' => [WormholeClass::HighSec]],
            ['name' => 'V283 - 0.0', 'signature' => WormholeSignature::V283, 'target_class' => WormholeTargetClass::N, 'spawn_areas' => [WormholeClass::HighSec]],
            ['name' => 'T458 - C12 Thera', 'signature' => WormholeSignature::T458, 'target_class' => WormholeTargetClass::C12, 'extra' => 'Thera', 'spawn_areas' => [WormholeClass::HighSec]],
            // Lowsec/Nullsec
            ['name' => 'N432 - C5', 'signature' => WormholeSignature::N432, 'target_class' => WormholeTargetClass::C5, 'spawn_areas' => [WormholeClass::LowSec, WormholeClass::NullSec]],
            ['name' => 'U319 - C6', 'signature' => WormholeSignature::U319, 'target_class' => WormholeTargetClass::C6, 'spawn_areas' => [WormholeClass::LowSec, WormholeClass::NullSec]],
            ['name' => 'B449 - H', 'signature' => WormholeSignature::B449, 'target_class' => WormholeTargetClass::H, 'spawn_areas' => [WormholeClass::LowSec, WormholeClass::NullSec]],
            ['name' => 'N944 - L', 'signature' => WormholeSignature::N944, 'target_class' => WormholeTargetClass::L, 'spawn_areas' => [WormholeClass::LowSec, WormholeClass::NullSec]],
            ['name' => 'S199 - 0.0', 'signature' => WormholeSignature::S199, 'target_class' => WormholeTargetClass::N, 'spawn_areas' => [WormholeClass::LowSec, WormholeClass::NullSec]],
            ['name' => 'M164 - C12 Thera', 'signature' => WormholeSignature::M164, 'target_class' => WormholeTargetClass::C12, 'extra' => 'Thera', 'spawn_areas' => [WormholeClass::LowSec]],
            ['name' => 'L031 - C12 Thera', 'signature' => WormholeSignature::L031, 'target_class' => WormholeTargetClass::C12, 'extra' => 'Thera', 'spawn_areas' => [WormholeClass::NullSec]],
        ];

        $category = SignatureCategory::where('code', 'wormhole')->firstOrFail();
        foreach ($wormholes as $wormhole) {
            SignatureType::updateOrCreate(
                ['name' => $wormhole['name'], 'signature_category_id' => $category->id],
                array_merge($wormhole, ['signature_category_id' => $category->id])
            );
        }
    }

    private function seedSpecialWormholes(): void
    {
        $wormholes = [
            // Pochven
            ['name' => 'R081 - C4', 'signature' => WormholeSignature::R081, 'target_class' => WormholeTargetClass::C4, 'spawn_areas' => [WormholeClass::Pochven]],
            ['name' => 'X450 - 0.0', 'signature' => WormholeSignature::X450, 'target_class' => WormholeTargetClass::N, 'spawn_areas' => [WormholeClass::Pochven]],
            ['name' => 'C729 - T Pochven', 'signature' => WormholeSignature::C729, 'target_class' => WormholeTargetClass::N, 'extra' => 'Pochven', 'spawn_areas' => [WormholeClass::HighSec, WormholeClass::LowSec, WormholeClass::NullSec]],
            ['name' => 'U372 - T Pochven', 'signature' => WormholeSignature::U372, 'target_class' => WormholeTargetClass::N, 'extra' => 'Pochven', 'spawn_areas' => [WormholeClass::NullSec]],
            ['name' => 'F216 - T Pochven', 'signature' => WormholeSignature::F216, 'target_class' => WormholeTargetClass::N, 'extra' => 'Pochven', 'spawn_areas' => [WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6]],
            // Thera
            ['name' => 'Q063 - H Thera', 'signature' => WormholeSignature::Q063, 'target_class' => WormholeTargetClass::H, 'extra' => 'Thera', 'spawn_areas' => [WormholeClass::C12]],
            ['name' => 'V898 - L Thera', 'signature' => WormholeSignature::V898, 'target_class' => WormholeTargetClass::L, 'extra' => 'Thera', 'spawn_areas' => [WormholeClass::C12]],
            ['name' => 'E587 - 0.0 Thera', 'signature' => WormholeSignature::E587, 'target_class' => WormholeTargetClass::N, 'extra' => 'Thera', 'spawn_areas' => [WormholeClass::C12]],
            // Special
            ['name' => 'J377 - L Turnur', 'signature' => WormholeSignature::J377, 'target_class' => WormholeTargetClass::L, 'extra' => 'Turnur', 'spawn_areas' => [WormholeClass::C1, WormholeClass::C2, WormholeClass::C3, WormholeClass::C4, WormholeClass::C5, WormholeClass::C6, WormholeClass::C13, WormholeClass::HighSec, WormholeClass::LowSec, WormholeClass::NullSec]],
            ['name' => 'U201 - L', 'signature' => WormholeSignature::U201, 'target_class' => WormholeTargetClass::L, 'spawn_areas' => [WormholeClass::C13]],
        ];

        $category = SignatureCategory::where('code', 'wormhole')->firstOrFail();
        foreach ($wormholes as $wormhole) {
            SignatureType::updateOrCreate(
                ['name' => $wormhole['name'], 'signature_category_id' => $category->id],
                array_merge($wormhole, ['signature_category_id' => $category->id])
            );
        }
    }
}
