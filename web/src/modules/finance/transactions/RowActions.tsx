import { MoreHorizontal } from "lucide-react";
import { useState } from "react";
import type { TransactionItem } from "@/api/finance";
import { MotionButton, Popover, PopoverContent, PopoverTrigger } from "@/design";
import { LinkTransferDialog } from "./LinkTransferDialog";

interface RowActionsProps {
  item: TransactionItem;
  names: ReadonlyMap<number, string>;
}

/**
 * The row's actions menu: an icon button that opens a menu with "Link as Transfer…", which
 * opens the dialog. From `md` the button appears on hover or focus of the row (the row is
 * `group/row`); below it is always shown. The dialog lives outside the menu so it survives the
 * menu closing.
 */
export function RowActions({ item, names }: RowActionsProps) {
  const [menuOpen, setMenuOpen] = useState(false);
  const [dialogOpen, setDialogOpen] = useState(false);
  return (
    <>
      <Popover open={menuOpen} onOpenChange={setMenuOpen}>
        <PopoverTrigger asChild>
          <MotionButton
            variant="ghost"
            size="icon-sm"
            aria-label="Actions for this Transaction"
            className="md:opacity-0 md:group-focus-within/row:opacity-100 md:group-hover/row:opacity-100 md:data-[state=open]:opacity-100"
          >
            <MoreHorizontal aria-hidden="true" className="size-4" />
          </MotionButton>
        </PopoverTrigger>
        <PopoverContent align="end" className="w-auto min-w-44 p-1">
          <MotionButton
            variant="ghost"
            size="sm"
            className="w-full justify-start"
            onClick={() => { setMenuOpen(false); setDialogOpen(true); }}
          >
            Link as Transfer…
          </MotionButton>
        </PopoverContent>
      </Popover>
      <LinkTransferDialog item={item} names={names} open={dialogOpen} onOpenChange={setDialogOpen} />
    </>
  );
}
