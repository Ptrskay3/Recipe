import {
  Slider,
  SliderMark,
  SliderTrack,
  SliderFilledTrack,
  Tooltip,
  SliderThumb,
  useMediaQuery,
} from '@chakra-ui/react';
import { useState } from 'react';

export function DurationSlider({ onChangeEnd }: { onChangeEnd?: (...args: any) => any }) {
  const [sliderValue, setSliderValue] = useState(0);
  const [showTooltip, setShowTooltip] = useState(false);
  const [isLargerThan900] = useMediaQuery('(min-width: 900px)');
  const suffix = isLargerThan900 ? 'mins' : 'm';
  return (
    <Slider
      id="slider"
      defaultValue={0}
      min={0}
      max={240}
      colorScheme="orange"
      onChange={(v) => setSliderValue(v)}
      onMouseEnter={() => setShowTooltip(true)}
      onMouseLeave={() => setShowTooltip(false)}
      onChangeEnd={onChangeEnd}
    >
      {isLargerThan900 ? (
        <SliderMark value={10} mt="1" ml="-2.5" fontSize="sm">
          10 {suffix}
        </SliderMark>
      ) : null}
      <SliderMark value={30} mt="1" ml="-2.5" fontSize="sm">
        30 {suffix}
      </SliderMark>
      <SliderMark value={60} mt="1" ml="-2.5" fontSize="sm">
        60 {suffix}
      </SliderMark>
      <SliderMark value={90} mt="1" ml="-2.5" fontSize="sm">
        90 {suffix}
      </SliderMark>
      <SliderMark value={120} mt="1" ml="-2.5" fontSize="sm">
        120 {suffix}
      </SliderMark>
      <SliderMark value={180} mt="1" ml="-2.5" fontSize="sm">
        180 {suffix}
      </SliderMark>
      <SliderTrack>
        <SliderFilledTrack />
      </SliderTrack>
      <Tooltip
        hasArrow
        bg="orange.400"
        color="white"
        placement="top"
        isOpen={showTooltip}
        label={`${sliderValue} minutes`}
      >
        <SliderThumb />
      </Tooltip>
    </Slider>
  );
}
