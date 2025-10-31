import { TSolarsystem } from '@/types/models';

export type TEveScoutConnection = {
    in_system: TSolarsystem;
    out_system: TSolarsystem;
    in_signature: string;
    out_signature: string;
    wormhole_type: string;
    life: string;
    mass: string;
    remaining_hours: number | null;
    created_at: string | null;
    jumps_from_selected: number | null;
};
