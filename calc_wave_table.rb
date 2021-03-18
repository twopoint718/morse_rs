half_crossing = 0

STDOUT.write("const WAV: [u8; 75] = [")
(0..2646).each do |x|
  sample = ((Math.sin(((600.0 * x) / 44100.0) * 2.0 * Math::PI) + 1.0) * 128.0).floor

  if sample < 128
    half_crossing = 1
  end
  if sample > 128 && half_crossing == 1
    STDOUT.write("#{sample}];")
    exit 0
  end

  STDOUT.write("#{sample}, ")
end
