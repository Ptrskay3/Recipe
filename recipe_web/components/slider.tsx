import {
  Slider,
  SliderMark,
  SliderTrack,
  SliderFilledTrack,
  Tooltip,
  SliderThumb,
} from '@chakra-ui/react';
import { useState } from 'react';

export function DurationSlider() {
  const [sliderValue, setSliderValue] = useState(5);
  const [showTooltip, setShowTooltip] = useState(false);
  return (
    <Slider
      id="slider"
      defaultValue={5}
      min={0}
      max={240}
      colorScheme="orange"
      onChange={(v) => setSliderValue(v)}
      onMouseEnter={() => setShowTooltip(true)}
      onMouseLeave={() => setShowTooltip(false)}
      onChangeEnd={(val) => console.log(val)} // TODO
    >
      <SliderMark value={10} mt="1" ml="-2.5" fontSize="sm">
        10 mins
      </SliderMark>
      <SliderMark value={30} mt="1" ml="-2.5" fontSize="sm">
        30 mins
      </SliderMark>
      <SliderMark value={60} mt="1" ml="-2.5" fontSize="sm">
        60 mins
      </SliderMark>
      <SliderMark value={90} mt="1" ml="-2.5" fontSize="sm">
        90 mins
      </SliderMark>
      <SliderMark value={120} mt="1" ml="-2.5" fontSize="sm">
        120 mins
      </SliderMark>
      <SliderMark value={180} mt="1" ml="-2.5" fontSize="sm">
        180 mins
      </SliderMark>
      <SliderTrack>
        <SliderFilledTrack />
      </SliderTrack>
      <Tooltip
        hasArrow
        bg="orange.500"
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
