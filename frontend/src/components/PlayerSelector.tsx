export type PlayerType = "human" | "bot";

interface Props {
  color: "White" | "Black";
  value: PlayerType;
  onChange: (value: PlayerType) => void;
}

export default function PlayerSelector({
  color,
  value,
  onChange,
}: Props) {
  return (
    <div className="player-section">
      <legend>{color}</legend>

      <label>
        <input
          type="radio"
          name={`${color.toLowerCase()}-player`}
          value="human"
          checked={value === "human"}
          onChange={() => onChange("human")}
        />
        Human
      </label>

      <label>
        <input
          type="radio"
          name={`${color.toLowerCase()}-player`}
          value="bot"
          checked={value === "bot"}
          onChange={() => onChange("bot")}
        />
        Bot
      </label>
    </div>
  );
}