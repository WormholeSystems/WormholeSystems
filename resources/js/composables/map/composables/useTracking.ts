import { createMapSolarsystem, createTracking, map_solarsystems, updateMapUserSettings } from '@/composables/map';
import { getSecurityClass } from '@/composables/map/utils/security';
import { useActiveMapCharacter } from '@/composables/useActiveMapCharacter';
import { useMapUserSettings } from '@/composables/useMapUserSettings';
import { useTrackingSystems } from '@/composables/useTrackingSystems';
import { TSolarsystem } from '@/pages/maps';
import { TSignature, TSolarsystemClass } from '@/types/models';
import { computed, onMounted, ref, watch } from 'vue';

export function useTracking() {
    const character = useActiveMapCharacter();
    const map_user_settings = useMapUserSettings();

    const is_tracking = computed(() => map_user_settings.value?.is_tracking && character.value && map_user_settings.value?.tracking_allowed);
    const is_tracking_allowed = computed(() => map_user_settings.value.tracking_allowed);
    const can_track = computed(() => character.value && map_user_settings.value.tracking_allowed);

    const { origin_map_solarsystem, target_solarsystem, update } = useTrackingSystems();

    const show_signature_modal = ref(false);
    const signatures = computed(() => origin_map_solarsystem.value?.signatures?.toSorted(sortSignatures).filter(isPossibleSignature));
    const existing_map_solarsystem = computed(() => map_solarsystems.value.find((s) => s.solarsystem_id === target_solarsystem.value?.id));
    const existing_connection = computed(() => {
        if (!existing_map_solarsystem.value) return null;
        return (
            signatures.value?.find(
                (s) =>
                    s.map_connection?.to_map_solarsystem_id === existing_map_solarsystem.value?.id ||
                    s.map_connection?.from_map_solarsystem_id === existing_map_solarsystem.value?.id,
            ) || null
        );
    });

    watch(
        () => character.value?.status?.solarsystem_id,
        (new_solarsystem_id, old_solarsystem_id) => {
            if (!map_user_settings.value.is_tracking) return;
            if (!new_solarsystem_id || !old_solarsystem_id) return;
            if (new_solarsystem_id === old_solarsystem_id) return;

            handleSolarsystemJump(old_solarsystem_id, new_solarsystem_id);
        },
    );

    onMounted(() => {
        if (!map_user_settings.value.is_tracking) return;

        addCurrentSolarsystemIfNotOnMap();
    });

    function handleSolarsystemJump(old_solarsystem_id: number | null, new_solarsystem_id: number) {
        const old_map_solarsystem = map_solarsystems.value.find((s) => s.solarsystem_id === old_solarsystem_id);
        if (!old_map_solarsystem) return;
        if (old_map_solarsystem.solarsystem_id === new_solarsystem_id) return;
        update(old_map_solarsystem.id, new_solarsystem_id, performJump);
    }

    function sortSignatures(a: TSignature, b: TSignature) {
        if (!a.signature_id || !b.signature_id) return 0;
        if (a.map_connection_id && !b.map_connection_id) return 1;
        if (!a.map_connection_id && b.map_connection_id) return -1;
        return a.signature_id?.localeCompare(b.signature_id);
    }

    function getTargetSolarsystemClass(system: TSolarsystem): TSolarsystemClass {
        if (system.class) return system.class;
        return getSecurityClass(system.security);
    }

    function isPossibleSignature(signature: TSignature): boolean {
        if (!signature.signature_type_id) return true;
        if (!target_solarsystem.value) return true;
        const solarsystem_class = getTargetSolarsystemClass(target_solarsystem.value);
        if (!solarsystem_class) return true;
        if (signature.signature_type?.target_class === String(solarsystem_class)) return true;
        return signature.signature_type?.target_class === null;
    }

    function performJump() {
        if (existing_connection.value?.map_connection_id) return;
        if (!signatures.value?.length || !map_user_settings.value.prompt_for_signature_enabled) {
            return createTracking(origin_map_solarsystem.value!.id, target_solarsystem.value!.id);
        }

        show_signature_modal.value = true;
    }

    function handleToggle() {
        if (!map_user_settings.value.tracking_allowed) return;

        updateMapUserSettings(map_user_settings.value, {
            is_tracking: !map_user_settings.value.is_tracking,
        });
    }

    function addCurrentSolarsystemIfNotOnMap() {
        const active_solarsystem = character.value?.status?.solarsystem;
        if (!active_solarsystem) return;

        if (isSolarsystemInMap(active_solarsystem.id)) return;

        createMapSolarsystem(active_solarsystem.id);
    }

    function isSolarsystemInMap(solarsystem_id: number): boolean {
        return map_solarsystems.value.some((s) => s.solarsystem_id === solarsystem_id);
    }

    function handleSelectSignature(signature_id: number | null) {
        show_signature_modal.value = false;
        if (!origin_map_solarsystem.value || !target_solarsystem.value) return;
        createTracking(origin_map_solarsystem.value.id, target_solarsystem.value.id, signature_id);
    }

    return {
        toggle: handleToggle,
        is_tracking,
        is_tracking_allowed,
        can_track,
        signatures,
        show_signature_modal,
        handleSelectSignature,
        origin_map_solarsystem,
        target_solarsystem,
    };
}
