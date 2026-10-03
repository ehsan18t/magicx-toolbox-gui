/** Joins name, description and info into one searchable string. */
export const FIELD_SEP = " | ";

export interface FieldBounds {
  nameEnd: number;
  descEnd: number;
}

/** Splits ranges over the joined string back into per-field ranges. */
export function fieldRanges(ranges: number[], { nameEnd, descEnd }: FieldBounds) {
  const nameRanges: number[] = [];
  const descriptionRanges: number[] = [];
  const infoRanges: number[] = [];

  const descStart = nameEnd + FIELD_SEP.length;
  const infoStart = descEnd + FIELD_SEP.length;

  for (let i = 0; i < ranges.length; i += 2) {
    const start = ranges[i];
    const end = ranges[i + 1];
    if (end <= nameEnd) {
      nameRanges.push(start, end);
    } else if (start >= infoStart) {
      infoRanges.push(start - infoStart, end - infoStart);
    } else if (start >= descStart && end <= descEnd) {
      descriptionRanges.push(start - descStart, end - descStart);
    } else {
      if (start < nameEnd) nameRanges.push(start, Math.min(end, nameEnd));
      if (start < descEnd && end > descStart) {
        descriptionRanges.push(Math.max(0, start - descStart), Math.min(end - descStart, descEnd - descStart));
      }
      if (end > infoStart) infoRanges.push(Math.max(0, start - infoStart), end - infoStart);
    }
  }
  return { nameRanges, descriptionRanges, infoRanges };
}
