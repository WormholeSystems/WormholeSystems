import {
    createMapSolarsystem,
    createTracking,
    formatBookmarkName,
    formatHomeBookmarkName,
    getSignatureIdShort,
    isWormholeSystem,
    map_solarsystems,
    suggestAlias,
    updateMapUserSettings,
} from '@/composables/map';
import { getSecurityClass } from '@/composables/map/utils/security';
import { useActiveMapCharacter } from '@/composables/useActiveMapCharacter';
import { useMapIgnoredSystems } from '@/composables/useMapIgnoredSystems';
import { useMapUserSettings } from '@/composables/useMapUserSettings';
import { useShowMap } from '@/composables/useShowMap';
import { useStaticData } from '@/composables/useStaticData';
import { useTrackingSystems } from '@/composables/useTrackingSystems';
import { TSolarsystem } from '@/pages/maps';
import { TLifetimeStatus, TMassStatus, TSignature, TSolarsystemClass } from '@/types/models';
import { computed, onMounted, ref, watch } from 'vue';
import { toast } from 'vue-sonner';

export function useTracking() {
    const character = useActiveMapCharacter();
    const map_user_settings = useMapUserSettings();
    const { isIgnored } = useMapIgnoredSystems();
    const page = useShowMap();
    const { staticData } = useStaticData();

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

    // Pre-fill the signature dialog's alias field. An alias the target already
    // carries on the map wins; otherwise we guess the next chain alias.
    const suggested_alias = computed(() => {
        if (existing_map_solarsystem.value?.alias) {
            return existing_map_solarsystem.value.alias;
        }

        if (!map_user_settings.value.suggest_alias_enabled) return null;

        const origin = origin_map_solarsystem.value;
        const target = target_solarsystem.value;
        if (!origin || !target) return null;

        return suggestAlias({
            parentAlias: origin.alias,
            targetIsWormhole: isWormholeSystem(target),
            originIsWormhole: isWormholeSystem(origin.solarsystem),
            aliases: map_solarsystems.value.map((s) => s.alias).filter((alias): alias is string => Boolean(alias)),
        });
    });

    watch(
        () => [character.value?.id, character.value?.status?.solarsystem_id] as const,
        ([new_character_id, new_solarsystem_id], [old_character_id, old_solarsystem_id]) => {
            if (!map_user_settings.value.is_tracking) return;
            if (!new_solarsystem_id || !old_solarsystem_id) return;
            if (new_solarsystem_id === old_solarsystem_id) return;
            // Only a single character moving between systems is a real jump. When
            // the active character is switched the watched system id also changes,
            // but that must not create a connection between the two characters' systems.
            if (new_character_id !== old_character_id) return;

            handleSolarsystemJump(old_solarsystem_id, new_solarsystem_id);
        },
    );

    onMounted(() => {
        if (!map_user_settings.value.is_tracking) return;

        addCurrentSolarsystemIfNotOnMap();
    });

    function handleSolarsystemJump(old_solarsystem_id: number | null, new_solarsystem_id: number) {
        if (isIgnored(new_solarsystem_id)) return;
        const old_map_solarsystem = map_solarsystems.value.find((s) => s.solarsystem_id === old_solarsystem_id);
        if (!old_map_solarsystem) return;
        if (old_map_solarsystem.solarsystem_id === new_solarsystem_id) return;
        update(old_map_solarsystem.id, new_solarsystem_id, performJump);
    }

    function isGateConnected(origin_solarsystem_id: number | null | undefined, target_solarsystem_id: number | null | undefined): boolean {
        if (!origin_solarsystem_id || !target_solarsystem_id) return false;
        return staticData.value?.connections[origin_solarsystem_id]?.includes(target_solarsystem_id) ?? false;
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
        const gate_connected = isGateConnected(origin_map_solarsystem.value?.solarsystem_id, target_solarsystem.value?.id);
        if (gate_connected || !signatures.value?.length || !map_user_settings.value.prompt_for_signature_enabled) {
            return createTracking(origin_map_solarsystem.value!.id, target_solarsystem.value!.id);
        }

        show_signature_modal.value = true;
    }

    function handleToggle() {
        if (!map_user_settings.value.tracking_allowed) return;

        updateMapUserSettings(page.props.map.slug, {
            is_tracking: !map_user_settings.value.is_tracking,
        });
    }

    function addCurrentSolarsystemIfNotOnMap() {
        const active_solarsystem_id = character.value?.status?.solarsystem_id;
        if (!active_solarsystem_id) return;

        if (isIgnored(active_solarsystem_id)) return;

        if (isSolarsystemInMap(active_solarsystem_id)) return;

        createMapSolarsystem(active_solarsystem_id);
    }

    function isSolarsystemInMap(solarsystem_id: number): boolean {
        return map_solarsystems.value.some((s) => s.solarsystem_id === solarsystem_id);
    }

    function handleSelectSignature(selection: {
        signatureId: number | null;
        alias: string | null;
        lifetime: TLifetimeStatus;
        massStatus: TMassStatus;
    }) {
        show_signature_modal.value = false;
        if (!origin_map_solarsystem.value || !target_solarsystem.value) return;
        copyConnectionBookmark(selection.signatureId, selection.alias);
        createTracking(origin_map_solarsystem.value.id, target_solarsystem.value.id, {
            signature_id: selection.signatureId,
            alias: selection.alias,
            lifetime: selection.lifetime,
            mass_status: selection.massStatus,
        });
    }

    // Copy the connection bookmark for the system we just jumped into, using the
    // same scheme as the connection context menu: the current system labelled
    // with the signature we used in the origin.
    function copyConnectionBookmark(signatureId: number | null, alias: string | null) {
        if (!map_user_settings.value.copy_bookmark_enabled) return;
        const target = target_solarsystem.value;
        if (!target) return;

        const signature = signatures.value?.find((s) => s.id === signatureId) ?? null;
        const name = formatHomeBookmarkName({ alias, solarsystem: target });

        navigator.clipboard.writeText(name);
        toast.success('Copied bookmark to clipboard', { description: name });
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
        existing_map_solarsystem,
        suggested_alias,
    };
}
