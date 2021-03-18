# Statically compute the "shape" of the wave over a full period and output an
# array definition suitable for a rust program (note: this will exit before the
# full 2646 samples)

half_crossing = 0

STDOUT.write("const WAV: [u8; 75] = [")
(0..2646).each do |x|
  sample = ((Math.sin(((588.0 * x) / 44100.0) * 2.0 * Math::PI) + 1.0) * 128.0).floor

  if sample < 128
    half_crossing = 1
  end
  if sample > 128 && half_crossing == 1
    STDOUT.write("#{sample}];\n")
    exit 0
  end

  STDOUT.write("#{sample}, ")
end
