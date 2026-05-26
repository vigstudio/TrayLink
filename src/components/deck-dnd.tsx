import {
  DndContext,
  KeyboardSensor,
  MouseSensor,
  TouchSensor,
  closestCenter,
  useSensor,
  useSensors,
  type DragEndEvent,
} from "@dnd-kit/core";
import {
  SortableContext,
  arrayMove,
  rectSortingStrategy,
  sortableKeyboardCoordinates,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import {
  ChevronDown,
  ChevronUp,
  Eye,
  EyeOff,
  GripVertical,
  ImagePlus,
  RotateCcw,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { DeckIcon } from "@/components/DeckIcon";
import { slotId, type DeckEditorItem } from "@/lib/remote-deck";

/** Sensors theo khuyến nghị dnd-kit: Mouse + Touch riêng, drag handle có touch-action: none */
export function useDeckSensors() {
  return useSensors(
    useSensor(MouseSensor, { activationConstraint: { distance: 8 } }),
    useSensor(TouchSensor, { activationConstraint: { delay: 200, tolerance: 8 } }),
    useSensor(KeyboardSensor, { coordinateGetter: sortableKeyboardCoordinates }),
  );
}

function SortablePreviewTile({
  item,
  onPickIcon,
}: {
  item: DeckEditorItem;
  onPickIcon: (item: DeckEditorItem) => void;
}) {
  const id = slotId(item.type, item.key);
  const {
    attributes,
    listeners,
    setNodeRef,
    setActivatorNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={`group relative flex select-none flex-col items-center gap-1.5 ${
        isDragging ? "z-10 opacity-50" : ""
      }`}
    >
      <div className="relative flex h-[72px] w-[72px] items-center justify-center rounded-xl border border-white/10 bg-[#22222e] shadow-inner">
        <button
          type="button"
          ref={setActivatorNodeRef}
          {...attributes}
          {...listeners}
          style={{ touchAction: "none" }}
          className="absolute left-1 top-1 z-10 flex h-6 w-6 cursor-grab items-center justify-center rounded bg-black/50 text-white/80 active:cursor-grabbing"
          aria-label={`Kéo để sắp xếp ${item.label}`}
        >
          <GripVertical className="size-3.5" />
        </button>

        <button
          type="button"
          onClick={() => onPickIcon(item)}
          title="Đổi icon"
          className="flex h-full w-full items-center justify-center"
        >
          <DeckIcon item={item} size="md" className="pointer-events-none h-12 w-12 border-0 bg-transparent" />
        </button>

        <span className="pointer-events-none absolute inset-x-1 bottom-1 rounded bg-black/60 px-1 py-0.5 text-[9px] text-white opacity-0 transition group-hover:opacity-100">
          Đổi icon
        </span>
      </div>
      <span className="max-w-[80px] truncate text-center text-[10px] text-white/60">{item.label}</span>
    </div>
  );
}

export function DeckPreviewSortable({
  items,
  onReorder,
  onPickIcon,
}: {
  items: DeckEditorItem[];
  onReorder: (nextItems: DeckEditorItem[]) => void;
  onPickIcon: (item: DeckEditorItem) => void;
}) {
  const visible = items.filter((item) => item.visible);
  const sensors = useDeckSensors();
  const sortableIds = visible.map((item) => slotId(item.type, item.key));

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    if (!over || active.id === over.id) return;

    const oldIndex = sortableIds.indexOf(String(active.id));
    const newIndex = sortableIds.indexOf(String(over.id));
    if (oldIndex < 0 || newIndex < 0) return;

    const reorderedVisible = arrayMove(visible, oldIndex, newIndex);
    const hidden = items.filter((item) => !item.visible);
    onReorder([...reorderedVisible, ...hidden]);
  };

  if (visible.length === 0) {
    return (
      <p className="rounded-xl border border-dashed p-6 text-center text-sm text-muted-foreground">
        Không có mục nào hiển thị trên Remote Deck
      </p>
    );
  }

  return (
    <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={handleDragEnd}>
      <div className="rounded-xl bg-[#0f0f12] p-4">
        <SortableContext items={sortableIds} strategy={rectSortingStrategy}>
          <div className="grid grid-cols-[repeat(auto-fill,minmax(72px,1fr))] gap-3">
            {visible.map((item) => (
              <SortablePreviewTile key={slotId(item.type, item.key)} item={item} onPickIcon={onPickIcon} />
            ))}
          </div>
        </SortableContext>
      </div>
    </DndContext>
  );
}

function SortableDeckRow({
  item,
  index,
  total,
  onMove,
  onToggleVisible,
  onPickIcon,
  onResetIcon,
}: {
  item: DeckEditorItem;
  index: number;
  total: number;
  onMove: (from: number, to: number) => void;
  onToggleVisible: (index: number) => void;
  onPickIcon: (item: DeckEditorItem) => void;
  onResetIcon: (item: DeckEditorItem) => void;
}) {
  const id = slotId(item.type, item.key);
  const {
    attributes,
    listeners,
    setNodeRef,
    setActivatorNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id });

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
  };

  return (
    <div
      ref={setNodeRef}
      style={style}
      className={`flex items-center gap-3 rounded-lg border bg-card px-3 py-2 ${
        item.visible ? "" : "opacity-50"
      } ${isDragging ? "z-10 opacity-60 shadow-md ring-1 ring-blue-400/40" : ""}`}
    >
      <button
        type="button"
        ref={setActivatorNodeRef}
        {...attributes}
        {...listeners}
        style={{ touchAction: "none" }}
        className="flex shrink-0 cursor-grab active:cursor-grabbing"
        aria-label={`Kéo để sắp xếp ${item.label}`}
      >
        <GripVertical className="size-4 text-muted-foreground" />
      </button>

      <DeckIcon item={item} />

      <div className="min-w-0 flex-1">
        <p className="truncate text-sm font-medium">{item.label}</p>
        <p className="truncate font-mono text-xs text-muted-foreground">{item.key}</p>
      </div>

      <div className="flex items-center gap-2">
        <Button variant="outline" size="sm" className="h-8 px-2" onClick={() => onPickIcon(item)}>
          <ImagePlus className="size-3.5" />
          Icon
        </Button>
        {item.customIcon && (
          <Button variant="ghost" size="sm" className="h-8 px-2" onClick={() => onResetIcon(item)}>
            <RotateCcw className="size-3.5" />
          </Button>
        )}

        <div className="flex items-center gap-2">
          {item.visible ? (
            <Eye className="size-4 text-muted-foreground" />
          ) : (
            <EyeOff className="size-4 text-muted-foreground" />
          )}
          <Switch checked={item.visible} onCheckedChange={() => onToggleVisible(index)} />
        </div>

        <div className="flex flex-col">
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            disabled={index === 0}
            onClick={() => onMove(index, index - 1)}
          >
            <ChevronUp className="size-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            disabled={index === total - 1}
            onClick={() => onMove(index, index + 1)}
          >
            <ChevronDown className="size-4" />
          </Button>
        </div>
      </div>
    </div>
  );
}

export function DeckSectionSortable({
  sectionItems,
  allItems,
  type,
  onSetItems,
  onPersist,
  onPickIcon,
  onResetIcon,
  reorderTypeItems,
  moveItem,
}: {
  sectionItems: DeckEditorItem[];
  allItems: DeckEditorItem[];
  type: "app" | "cmd";
  onSetItems: (items: DeckEditorItem[]) => void;
  onPersist: () => void;
  onPickIcon: (item: DeckEditorItem) => void;
  onResetIcon: (item: DeckEditorItem) => void;
  reorderTypeItems: (
    allItems: DeckEditorItem[],
    type: "app" | "cmd",
    orderedKeys: string[],
  ) => DeckEditorItem[];
  moveItem: <T>(items: T[], from: number, to: number) => T[];
}) {
  const sensors = useDeckSensors();
  const sortableIds = sectionItems.map((item) => slotId(item.type, item.key));

  const applyMove = (fromKey: string, toIndex: number) => {
    const currentSection = allItems.filter((item) => item.type === type);
    const fromIndex = currentSection.findIndex((item) => item.key === fromKey);
    if (fromIndex < 0 || fromIndex === toIndex) return;

    const reordered = moveItem(currentSection, fromIndex, toIndex);
    onSetItems(reorderTypeItems(allItems, type, reordered.map((item) => item.key)));
  };

  const handleToggleVisible = (index: number) => {
    const key = sectionItems[index]?.key;
    if (!key) return;
    onSetItems(
      allItems.map((item) =>
        item.type === type && item.key === key ? { ...item, visible: !item.visible } : item,
      ),
    );
    onPersist();
  };

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;
    if (!over || active.id === over.id) return;

    const oldIndex = sortableIds.indexOf(String(active.id));
    const newIndex = sortableIds.indexOf(String(over.id));
    if (oldIndex < 0 || newIndex < 0) return;

    const reordered = arrayMove(sectionItems, oldIndex, newIndex);
    onSetItems(reorderTypeItems(allItems, type, reordered.map((item) => item.key)));
    onPersist();
  };

  return (
    <DndContext sensors={sensors} collisionDetection={closestCenter} onDragEnd={handleDragEnd}>
      <SortableContext items={sortableIds} strategy={verticalListSortingStrategy}>
        <div className="space-y-2">
          {sectionItems.map((item, index) => (
            <SortableDeckRow
              key={slotId(item.type, item.key)}
              item={item}
              index={index}
              total={sectionItems.length}
              onMove={(from, to) => {
                const fromKey = sectionItems[from]?.key;
                if (!fromKey) return;
                applyMove(fromKey, to);
                onPersist();
              }}
              onToggleVisible={handleToggleVisible}
              onPickIcon={onPickIcon}
              onResetIcon={onResetIcon}
            />
          ))}
        </div>
      </SortableContext>
    </DndContext>
  );
}
