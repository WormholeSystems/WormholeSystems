import AllianceLogo from '@/components/images/AllianceLogo.vue';
import CharacterImage from '@/components/images/CharacterImage.vue';
import CorporationLogo from '@/components/images/CorporationLogo.vue';
import FactionLogo from '@/components/images/FactionLogo.vue';
import TypeImage from '@/components/images/TypeImage.vue';

type TImageSize = 32 | 64 | 128 | 256 | 512;
type TTypeImageVariants = 'icon' | 'render';

export { AllianceLogo, CharacterImage, CorporationLogo, FactionLogo, TypeImage };

export type { TImageSize, TTypeImageVariants };
