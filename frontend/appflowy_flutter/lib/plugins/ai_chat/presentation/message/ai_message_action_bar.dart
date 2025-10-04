import 'dart:async';
import 'dart:convert';

import 'package:appflowy/ai/ai.dart';
import 'package:appflowy/generated/flowy_svgs.g.dart';
import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/plugins/ai_chat/application/chat_ai_message_bloc.dart';
import 'package:appflowy/plugins/ai_chat/application/chat_edit_document_service.dart';
import 'package:appflowy/plugins/document/application/prelude.dart';
import 'package:appflowy/plugins/document/presentation/editor_plugins/copy_and_paste/clipboard_service.dart';
import 'package:appflowy/shared/markdown_to_document.dart';
import 'package:appflowy/shared/patterns/common_patterns.dart';
import 'package:appflowy/startup/startup.dart';
import 'package:appflowy/util/theme_extension.dart';
import 'package:appflowy/workspace/application/sidebar/space/space_bloc.dart';
import 'package:appflowy/workspace/application/tabs/tabs_bloc.dart';
import 'package:appflowy/workspace/application/view/prelude.dart';
import 'package:appflowy/workspace/application/view/view_ext.dart';
import 'package:appflowy/workspace/presentation/home/menu/view/view_item.dart';
import 'package:appflowy/workspace/presentation/widgets/dialogs.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/protobuf.dart';
import 'package:appflowy_backend/protobuf/flowy-folder/protobuf.dart';
import 'package:appflowy_editor/appflowy_editor.dart';
import 'package:appflowy_result/appflowy_result.dart';
import 'package:appflowy_ui/appflowy_ui.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flowy_infra/theme_extension.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_chat_core/flutter_chat_core.dart';

import '../layout_define.dart';
import 'message_util.dart';
import '../execution_log_viewer.dart';
import '../../application/execution_log_bloc.dart';
import '../../application/chat_entity.dart';

class AIMessageActionBar extends StatefulWidget {
  const AIMessageActionBar({
    super.key,
    required this.message,
    required this.showDecoration,
    this.onRegenerate,
    this.onChangeFormat,
    this.onChangeModel,
    this.onOverrideVisibility,
  });

  final Message message;
  final bool showDecoration;
  final void Function()? onRegenerate;
  final void Function(PredefinedFormat)? onChangeFormat;
  final void Function(AIModelPB)? onChangeModel;
  final void Function(bool)? onOverrideVisibility;

  @override
  State<AIMessageActionBar> createState() => _AIMessageActionBarState();
}

class _AIMessageActionBarState extends State<AIMessageActionBar> {
  final popoverMutex = PopoverMutex();

  @override
  Widget build(BuildContext context) {
    final isLightMode = Theme.of(context).isLightMode;

    final child = SeparatedRow(
      mainAxisSize: MainAxisSize.min,
      separatorBuilder: () => const HSpace(8.0),
      children: _buildChildren(),
    );

    return widget.showDecoration
        ? Container(
            padding: DesktopAIChatSizes.messageHoverActionBarPadding,
            decoration: BoxDecoration(
              borderRadius: DesktopAIChatSizes.messageHoverActionBarRadius,
              border: Border.all(
                color: isLightMode
                    ? const Color(0x1F1F2329)
                    : Theme.of(context).dividerColor,
                strokeAlign: BorderSide.strokeAlignOutside,
              ),
              color: Theme.of(context).cardColor,
              boxShadow: [
                BoxShadow(
                  offset: const Offset(0, 1),
                  blurRadius: 2,
                  spreadRadius: -2,
                  color: isLightMode
                      ? const Color(0x051F2329)
                      : Theme.of(context).shadowColor.withValues(alpha: 0.02),
                ),
                BoxShadow(
                  offset: const Offset(0, 2),
                  blurRadius: 4,
                  color: isLightMode
                      ? const Color(0x051F2329)
                      : Theme.of(context).shadowColor.withValues(alpha: 0.02),
                ),
                BoxShadow(
                  offset: const Offset(0, 2),
                  blurRadius: 8,
                  spreadRadius: 2,
                  color: isLightMode
                      ? const Color(0x051F2329)
                      : Theme.of(context).shadowColor.withValues(alpha: 0.02),
                ),
              ],
            ),
            child: child,
          )
        : child;
  }

  List<Widget> _buildChildren() {
    return [
      CopyButton(
        isInHoverBar: widget.showDecoration,
        textMessage: widget.message as TextMessage,
      ),
      RegenerateButton(
        isInHoverBar: widget.showDecoration,
        onTap: () => widget.onRegenerate?.call(),
      ),
      ChangeFormatButton(
        isInHoverBar: widget.showDecoration,
        onRegenerate: widget.onChangeFormat,
        popoverMutex: popoverMutex,
        onOverrideVisibility: widget.onOverrideVisibility,
      ),
      ChangeModelButton(
        isInHoverBar: widget.showDecoration,
        onRegenerate: widget.onChangeModel,
        popoverMutex: popoverMutex,
        onOverrideVisibility: widget.onOverrideVisibility,
      ),
      ExecutionLogButton(
        isInHoverBar: widget.showDecoration,
        message: widget.message as TextMessage,
        popoverMutex: popoverMutex,
        onOverrideVisibility: widget.onOverrideVisibility,
      ),
      SaveToPageButton(
        textMessage: widget.message as TextMessage,
        isInHoverBar: widget.showDecoration,
        popoverMutex: popoverMutex,
        onOverrideVisibility: widget.onOverrideVisibility,
      ),
    ];
  }
}

class CopyButton extends StatelessWidget {
  const CopyButton({
    super.key,
    required this.isInHoverBar,
    required this.textMessage,
  });

  final bool isInHoverBar;
  final TextMessage textMessage;

  @override
  Widget build(BuildContext context) {
    return FlowyTooltip(
      message: LocaleKeys.settings_menu_clickToCopy.tr(),
      child: FlowyIconButton(
        width: DesktopAIChatSizes.messageActionBarIconSize,
        hoverColor: AFThemeExtension.of(context).lightGreyHover,
        radius: isInHoverBar
            ? DesktopAIChatSizes.messageHoverActionBarIconRadius
            : DesktopAIChatSizes.messageActionBarIconRadius,
        icon: FlowySvg(
          FlowySvgs.copy_s,
          color: Theme.of(context).hintColor,
          size: const Size.square(16),
        ),
        onPressed: () async {
          final messageText = textMessage.text.trim();
          final document = customMarkdownToDocument(
            messageText,
            tableWidth: 250.0,
          );
          await getIt<ClipboardService>().setData(
            ClipboardServiceData(
              plainText: _stripMarkdownIfNecessary(messageText),
              inAppJson: jsonEncode(document.toJson()),
            ),
          );
          if (context.mounted) {
            showToastNotification(
              message: LocaleKeys.message_copy_success.tr(),
            );
          }
        },
      ),
    );
  }

  String _stripMarkdownIfNecessary(String plainText) {
    // match and capture inner url as group
    final matches = singleLineMarkdownImageRegex.allMatches(plainText);

    if (matches.length != 1) {
      return plainText;
    }

    return matches.first[1] ?? plainText;
  }
}

class RegenerateButton extends StatelessWidget {
  const RegenerateButton({
    super.key,
    required this.isInHoverBar,
    required this.onTap,
  });

  final bool isInHoverBar;
  final void Function() onTap;

  @override
  Widget build(BuildContext context) {
    return FlowyTooltip(
      message: LocaleKeys.chat_regenerate.tr(),
      child: FlowyIconButton(
        width: DesktopAIChatSizes.messageActionBarIconSize,
        hoverColor: AFThemeExtension.of(context).lightGreyHover,
        radius: isInHoverBar
            ? DesktopAIChatSizes.messageHoverActionBarIconRadius
            : DesktopAIChatSizes.messageActionBarIconRadius,
        icon: FlowySvg(
          FlowySvgs.ai_try_again_s,
          color: Theme.of(context).hintColor,
          size: const Size.square(16),
        ),
        onPressed: onTap,
      ),
    );
  }
}

class ChangeFormatButton extends StatefulWidget {
  const ChangeFormatButton({
    super.key,
    required this.isInHoverBar,
    this.popoverMutex,
    this.onRegenerate,
    this.onOverrideVisibility,
  });

  final bool isInHoverBar;
  final PopoverMutex? popoverMutex;
  final void Function(PredefinedFormat)? onRegenerate;
  final void Function(bool)? onOverrideVisibility;

  @override
  State<ChangeFormatButton> createState() => _ChangeFormatButtonState();
}

class _ChangeFormatButtonState extends State<ChangeFormatButton> {
  final popoverController = PopoverController();

  @override
  Widget build(BuildContext context) {
    return AppFlowyPopover(
      controller: popoverController,
      mutex: widget.popoverMutex,
      triggerActions: PopoverTriggerFlags.none,
      margin: EdgeInsets.zero,
      offset: Offset(0, widget.isInHoverBar ? 8 : 4),
      direction: PopoverDirection.bottomWithLeftAligned,
      constraints: const BoxConstraints(),
      onClose: () => widget.onOverrideVisibility?.call(false),
      child: buildButton(context),
      popupBuilder: (_) => BlocProvider.value(
        value: context.read<AIPromptInputBloc>(),
        child: _ChangeFormatPopoverContent(
          onRegenerate: widget.onRegenerate,
        ),
      ),
    );
  }

  Widget buildButton(BuildContext context) {
    return FlowyTooltip(
      message: LocaleKeys.chat_changeFormat_actionButton.tr(),
      child: FlowyIconButton(
        width: 32.0,
        height: DesktopAIChatSizes.messageActionBarIconSize,
        hoverColor: AFThemeExtension.of(context).lightGreyHover,
        radius: widget.isInHoverBar
            ? DesktopAIChatSizes.messageHoverActionBarIconRadius
            : DesktopAIChatSizes.messageActionBarIconRadius,
        icon: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            FlowySvg(
              FlowySvgs.ai_retry_font_s,
              color: Theme.of(context).hintColor,
              size: const Size.square(16),
            ),
            FlowySvg(
              FlowySvgs.ai_source_drop_down_s,
              color: Theme.of(context).hintColor,
              size: const Size.square(8),
            ),
          ],
        ),
        onPressed: () {
          widget.onOverrideVisibility?.call(true);
          popoverController.show();
        },
      ),
    );
  }
}

class _ChangeFormatPopoverContent extends StatefulWidget {
  const _ChangeFormatPopoverContent({
    this.onRegenerate,
  });

  final void Function(PredefinedFormat)? onRegenerate;

  @override
  State<_ChangeFormatPopoverContent> createState() =>
      _ChangeFormatPopoverContentState();
}

class _ChangeFormatPopoverContentState
    extends State<_ChangeFormatPopoverContent> {
  PredefinedFormat? predefinedFormat;

  @override
  Widget build(BuildContext context) {
    final isLightMode = Theme.of(context).isLightMode;
    return Container(
      padding: const EdgeInsets.all(2.0),
      decoration: BoxDecoration(
        borderRadius: DesktopAIChatSizes.messageHoverActionBarRadius,
        border: Border.all(
          color: isLightMode
              ? const Color(0x1F1F2329)
              : Theme.of(context).dividerColor,
          strokeAlign: BorderSide.strokeAlignOutside,
        ),
        color: Theme.of(context).cardColor,
        boxShadow: [
          BoxShadow(
            offset: const Offset(0, 1),
            blurRadius: 2,
            spreadRadius: -2,
            color: isLightMode
                ? const Color(0x051F2329)
                : Theme.of(context).shadowColor.withValues(alpha: 0.02),
          ),
          BoxShadow(
            offset: const Offset(0, 2),
            blurRadius: 4,
            color: isLightMode
                ? const Color(0x051F2329)
                : Theme.of(context).shadowColor.withValues(alpha: 0.02),
          ),
          BoxShadow(
            offset: const Offset(0, 2),
            blurRadius: 8,
            spreadRadius: 2,
            color: isLightMode
                ? const Color(0x051F2329)
                : Theme.of(context).shadowColor.withValues(alpha: 0.02),
          ),
        ],
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          BlocBuilder<AIPromptInputBloc, AIPromptInputState>(
            builder: (context, state) {
              return ChangeFormatBar(
                spacing: 2.0,
                showImageFormats: state.modelState.type.isCloud,
                predefinedFormat: predefinedFormat,
                onSelectPredefinedFormat: (format) {
                  setState(() => predefinedFormat = format);
                },
              );
            },
          ),
          const HSpace(4.0),
          FlowyTooltip(
            message: LocaleKeys.chat_changeFormat_confirmButton.tr(),
            child: MouseRegion(
              cursor: SystemMouseCursors.click,
              child: GestureDetector(
                behavior: HitTestBehavior.opaque,
                onTap: () {
                  if (predefinedFormat != null) {
                    widget.onRegenerate?.call(predefinedFormat!);
                  }
                },
                child: SizedBox.square(
                  dimension: DesktopAIPromptSizes.predefinedFormatButtonHeight,
                  child: Center(
                    child: FlowySvg(
                      FlowySvgs.ai_retry_filled_s,
                      color: Theme.of(context).colorScheme.primary,
                      size: const Size.square(20),
                    ),
                  ),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class ChangeModelButton extends StatefulWidget {
  const ChangeModelButton({
    super.key,
    required this.isInHoverBar,
    this.popoverMutex,
    this.onRegenerate,
    this.onOverrideVisibility,
  });

  final bool isInHoverBar;
  final PopoverMutex? popoverMutex;
  final void Function(AIModelPB)? onRegenerate;
  final void Function(bool)? onOverrideVisibility;

  @override
  State<ChangeModelButton> createState() => _ChangeModelButtonState();
}

class _ChangeModelButtonState extends State<ChangeModelButton> {
  final popoverController = PopoverController();

  @override
  Widget build(BuildContext context) {
    return AppFlowyPopover(
      controller: popoverController,
      mutex: widget.popoverMutex,
      triggerActions: PopoverTriggerFlags.none,
      margin: EdgeInsets.zero,
      offset: Offset(8, 0),
      direction: PopoverDirection.rightWithBottomAligned,
      constraints: BoxConstraints(maxWidth: 250, maxHeight: 600),
      onClose: () => widget.onOverrideVisibility?.call(false),
      child: buildButton(context),
      popupBuilder: (_) {
        final bloc = context.read<AIPromptInputBloc>();
        final (models, _) = bloc.aiModelStateNotifier.getModelSelection();
        return SelectModelPopoverContent(
          models: models,
          selectedModel: null,
          onSelectModel: widget.onRegenerate,
        );
      },
    );
  }

  Widget buildButton(BuildContext context) {
    return FlowyTooltip(
      message: LocaleKeys.chat_switchModel_label.tr(),
      child: FlowyIconButton(
        width: 32.0,
        height: DesktopAIChatSizes.messageActionBarIconSize,
        hoverColor: AFThemeExtension.of(context).lightGreyHover,
        radius: widget.isInHoverBar
            ? DesktopAIChatSizes.messageHoverActionBarIconRadius
            : DesktopAIChatSizes.messageActionBarIconRadius,
        icon: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            FlowySvg(
              FlowySvgs.ai_sparks_s,
              color: Theme.of(context).hintColor,
              size: const Size.square(16),
            ),
            FlowySvg(
              FlowySvgs.ai_source_drop_down_s,
              color: Theme.of(context).hintColor,
              size: const Size.square(8),
            ),
          ],
        ),
        onPressed: () {
          widget.onOverrideVisibility?.call(true);
          popoverController.show();
        },
      ),
    );
  }
}

class SaveToPageButton extends StatefulWidget {
  const SaveToPageButton({
    super.key,
    required this.textMessage,
    required this.isInHoverBar,
    this.popoverMutex,
    this.onOverrideVisibility,
  });

  final TextMessage textMessage;
  final bool isInHoverBar;
  final PopoverMutex? popoverMutex;
  final void Function(bool)? onOverrideVisibility;

  @override
  State<SaveToPageButton> createState() => _SaveToPageButtonState();
}

class _SaveToPageButtonState extends State<SaveToPageButton> {
  final popoverController = PopoverController();

  @override
  Widget build(BuildContext context) {
    return ViewSelector(
      viewSelectorCubit: BlocProvider(
        create: (context) => ViewSelectorCubit(
          getIgnoreViewType: (item) {
            final view = item.view;

            if (view.isSpace) {
              return IgnoreViewType.none;
            }
            if (view.layout != ViewLayoutPB.Document) {
              return IgnoreViewType.hide;
            }

            return IgnoreViewType.none;
          },
        ),
      ),
      child: BlocSelector<SpaceBloc, SpaceState, ViewPB?>(
        selector: (state) => state.currentSpace,
        builder: (context, spaceView) {
          return AppFlowyPopover(
            controller: popoverController,
            triggerActions: PopoverTriggerFlags.none,
            margin: EdgeInsets.zero,
            mutex: widget.popoverMutex,
            offset: const Offset(8, 0),
            direction: PopoverDirection.rightWithBottomAligned,
            constraints: const BoxConstraints.tightFor(width: 300, height: 400),
            onClose: () {
              if (spaceView != null) {
                context
                    .read<ViewSelectorCubit>()
                    .refreshSources([spaceView], spaceView);
              }
              widget.onOverrideVisibility?.call(false);
            },
            child: buildButton(context, spaceView),
            popupBuilder: (_) => buildPopover(context),
          );
        },
      ),
    );
  }

  Widget buildButton(BuildContext context, ViewPB? spaceView) {
    return FlowyTooltip(
      message: LocaleKeys.chat_addToPageButton.tr(),
      child: FlowyIconButton(
        width: DesktopAIChatSizes.messageActionBarIconSize,
        hoverColor: AFThemeExtension.of(context).lightGreyHover,
        radius: widget.isInHoverBar
            ? DesktopAIChatSizes.messageHoverActionBarIconRadius
            : DesktopAIChatSizes.messageActionBarIconRadius,
        icon: FlowySvg(
          FlowySvgs.ai_add_to_page_s,
          color: Theme.of(context).hintColor,
          size: const Size.square(16),
        ),
        onPressed: () async {
          final documentId = getOpenedDocumentId();
          if (documentId != null) {
            await onAddToExistingPage(context, documentId);
            await forceReload(documentId);
            await Future.delayed(const Duration(milliseconds: 500));
            await updateSelection(documentId);
          } else {
            widget.onOverrideVisibility?.call(true);
            if (spaceView != null) {
              unawaited(
                context
                    .read<ViewSelectorCubit>()
                    .refreshSources([spaceView], spaceView),
              );
            }
            popoverController.show();
          }
        },
      ),
    );
  }

  Widget buildPopover(BuildContext context) {
    return BlocProvider.value(
      value: context.read<ViewSelectorCubit>(),
      child: SaveToPagePopoverContent(
        onAddToNewPage: (parentViewId) {
          addMessageToNewPage(context, parentViewId);
          popoverController.close();
        },
        onAddToExistingPage: (documentId) async {
          popoverController.close();
          final view = await onAddToExistingPage(context, documentId);

          if (context.mounted) {
            openPageFromMessage(context, view);
          }
          await Future.delayed(const Duration(milliseconds: 500));
          await updateSelection(documentId);
        },
      ),
    );
  }

  Future<ViewPB?> onAddToExistingPage(
    BuildContext context,
    String documentId,
  ) async {
    await ChatEditDocumentService.addMessagesToPage(
      documentId,
      [widget.textMessage],
    );
    await Future.delayed(const Duration(milliseconds: 500));
    final view = await ViewBackendService.getView(documentId).toNullable();
    if (context.mounted) {
      showSaveMessageSuccessToast(context, view);
    }
    return view;
  }

  void addMessageToNewPage(BuildContext context, String parentViewId) async {
    final chatView = await ViewBackendService.getView(
      context.read<ChatAIMessageBloc>().chatId,
    ).toNullable();
    if (chatView != null) {
      final newView = await ChatEditDocumentService.saveMessagesToNewPage(
        chatView.nameOrDefault,
        parentViewId,
        [widget.textMessage],
      );

      if (context.mounted) {
        showSaveMessageSuccessToast(context, newView);
        openPageFromMessage(context, newView);
      }
    }
  }

  Future<void> forceReload(String documentId) async {
    final bloc = DocumentBloc.findOpen(documentId);
    if (bloc == null) {
      return;
    }
    await bloc.forceReloadDocumentState();
  }

  Future<void> updateSelection(String documentId) async {
    final bloc = DocumentBloc.findOpen(documentId);
    if (bloc == null) {
      return;
    }
    await bloc.forceReloadDocumentState();
    final editorState = bloc.state.editorState;
    final lastNodePath = editorState?.getLastSelectable()?.$1.path;
    if (editorState == null || lastNodePath == null) {
      return;
    }
    unawaited(
      editorState.updateSelectionWithReason(
        Selection.collapsed(Position(path: lastNodePath)),
      ),
    );
  }

  String? getOpenedDocumentId() {
    final pageManager = getIt<TabsBloc>().state.currentPageManager;
    if (!pageManager.showSecondaryPluginNotifier.value) {
      return null;
    }
    return pageManager.secondaryNotifier.plugin.id;
  }
}

class SaveToPagePopoverContent extends StatelessWidget {
  const SaveToPagePopoverContent({
    super.key,
    required this.onAddToNewPage,
    required this.onAddToExistingPage,
  });

  final void Function(String) onAddToNewPage;
  final void Function(String) onAddToExistingPage;

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<ViewSelectorCubit, ViewSelectorState>(
      builder: (context, state) {
        final theme = AppFlowyTheme.of(context);

        return Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(
              height: 24,
              margin: const EdgeInsets.fromLTRB(12, 8, 12, 4),
              child: Align(
                alignment: AlignmentDirectional.centerStart,
                child: Text(
                  LocaleKeys.chat_addToPageTitle.tr(),
                  style: theme.textStyle.caption
                      .standard(color: theme.textColorScheme.secondary),
                ),
              ),
            ),
            Padding(
              padding: const EdgeInsets.only(left: 12, right: 12, bottom: 8),
              child: AFTextField(
                controller:
                    context.read<ViewSelectorCubit>().filterTextController,
                hintText: LocaleKeys.search_label.tr(),
                size: AFTextFieldSize.m,
              ),
            ),
            AFDivider(
              startIndent: theme.spacing.l,
              endIndent: theme.spacing.l,
            ),
            Expanded(
              child: ListView(
                shrinkWrap: true,
                padding: const EdgeInsets.fromLTRB(8, 4, 8, 12),
                children: _buildVisibleSources(context, state).toList(),
              ),
            ),
          ],
        );
      },
    );
  }

  Iterable<Widget> _buildVisibleSources(
    BuildContext context,
    ViewSelectorState state,
  ) {
    return state.visibleSources.map(
      (e) => ViewSelectorTreeItem(
        key: ValueKey(
          'save_to_page_tree_item_${e.view.id}',
        ),
        viewSelectorItem: e,
        level: 0,
        isDescendentOfSpace: e.view.isSpace,
        isSelectedSection: false,
        showCheckbox: false,
        showSaveButton: true,
        onSelected: (source) {
          if (source.view.isSpace) {
            onAddToNewPage(source.view.id);
          } else {
            onAddToExistingPage(source.view.id);
          }
        },
        onAdd: (source) {
          onAddToNewPage(source.view.id);
        },
        height: 30.0,
      ),
    );
  }
}

/// 执行日志按钮组件
class ExecutionLogButton extends StatefulWidget {
  const ExecutionLogButton({
    super.key,
    required this.isInHoverBar,
    required this.message,
    required this.popoverMutex,
    this.onOverrideVisibility,
  });

  final bool isInHoverBar;
  final TextMessage message;
  final PopoverMutex popoverMutex;
  final void Function(bool)? onOverrideVisibility;

  @override
  State<ExecutionLogButton> createState() => _ExecutionLogButtonState();
}

class _ExecutionLogButtonState extends State<ExecutionLogButton> {
  final PopoverController _popoverController = PopoverController();
  ExecutionLogBloc? _executionLogBloc;
  bool _isLoadingLogs = false; // 防止重复加载

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    
    // ✅ 在这里创建 Bloc（只创建一次）
    if (_executionLogBloc == null) {
      final chatId = context.read<ChatAIMessageBloc>().chatId;
      final questionIdRaw = widget.message.metadata?[messageQuestionIdKey];
      final questionId = questionIdRaw?.toString() ?? widget.message.id;
      
      print('🔍 [ExecutionLogButton] Creating ExecutionLogBloc in didChangeDependencies');
      print('🔍 [ExecutionLogButton] chatId: $chatId');
      print('🔍 [ExecutionLogButton] questionId: $questionId');
      
      _executionLogBloc = ExecutionLogBloc(
        sessionId: chatId,
        messageId: questionId,
      );
      print('🔍 [ExecutionLogButton] Created bloc hashCode: ${_executionLogBloc.hashCode}');
    }
  }

  @override
  void dispose() {
    print('🔍 [ExecutionLogButton] 🔴 DISPOSING - state hashCode: ${hashCode}');
    print('🔍 [ExecutionLogButton] 🔴 Bloc hashCode: ${_executionLogBloc?.hashCode}');
    print('🔍 [ExecutionLogButton] 🔴 Bloc isClosed: ${_executionLogBloc?.isClosed}');
    _popoverController.close();
    
    // ⚠️ 延迟关闭 Bloc，给异步操作足够时间完成
    // 这样可以避免在等待后端响应时 Bloc 被提前关闭
    Future.delayed(const Duration(milliseconds: 500), () {
      print('🔍 [ExecutionLogButton] 🔴 Delayed closing bloc');
      _executionLogBloc?.close();
    });
    
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    // ✅ 确保 Bloc 已创建
    if (_executionLogBloc == null) {
      print('🔍 [ExecutionLogButton] build: Bloc not ready yet');
      return const SizedBox.shrink();
    }
    
    print('🔍 [ExecutionLogButton] build: Bloc ready (hashCode: ${_executionLogBloc.hashCode}, isClosed: ${_executionLogBloc!.isClosed})');
    
    // ✅ 在外层获取真正的屏幕尺寸
    final screenSize = MediaQuery.of(context).size;
    print('🔍 [ExecutionLogButton] Screen size: ${screenSize.width} x ${screenSize.height}');
    
    // ✅ 计算 Popover 的约束尺寸
    final popoverWidth = (screenSize.width * 0.50).clamp(700.0, 1000.0);
    final popoverHeight = (screenSize.height * 0.75).clamp(500.0, 700.0);
    print('🔍 [ExecutionLogButton] Popover constraints: ${popoverWidth} x ${popoverHeight}');
    
    return AppFlowyPopover(
      controller: _popoverController,
      mutex: widget.popoverMutex,
      direction: PopoverDirection.bottomWithLeftAligned,
      offset: const Offset(-300, 10),
      // ⚠️ 关键修复：显式设置 constraints，覆盖默认的 240px 宽度限制！
      constraints: BoxConstraints(
        minWidth: popoverWidth,
        maxWidth: popoverWidth,
        minHeight: popoverHeight,
        maxHeight: popoverHeight,
      ),
      onOpen: () {
        print('🔍 [ExecutionLogButton] 🟢 Popover opened');
        print('🔍 [ExecutionLogButton] 🟢 _isLoadingLogs: $_isLoadingLogs');
        print('🔍 [ExecutionLogButton] 🟢 Bloc status: ${_executionLogBloc == null ? "NULL" : (_executionLogBloc!.isClosed ? "CLOSED" : "OPEN")}');
        print('🔍 [ExecutionLogButton] 🟢 Bloc hashCode: ${_executionLogBloc?.hashCode}');
        
        widget.onOverrideVisibility?.call(true);
        
        // ⚠️ 防止重复加载
        if (_isLoadingLogs) {
          print('🔍 [ExecutionLogButton] ⚠️ Already loading logs, skipping...');
          return;
        }
        
        // ✅ 加载日志（Bloc 已在 didChangeDependencies 中创建）
        if (_executionLogBloc != null && !_executionLogBloc!.isClosed) {
          print('🔍 [ExecutionLogButton] 🟢 Adding loadLogs event to bloc');
          _isLoadingLogs = true;
          _executionLogBloc!.add(const ExecutionLogEvent.loadLogs());
          print('🔍 [ExecutionLogButton] 🟢 loadLogs event added');
          
          // 500ms 后重置标志，允许再次加载
          Future.delayed(const Duration(milliseconds: 500), () {
            if (mounted) {
              _isLoadingLogs = false;
              print('🔍 [ExecutionLogButton] 🟢 Reset _isLoadingLogs flag');
            }
          });
        } else {
          print('🔍 [ExecutionLogButton] ⚠️ Cannot load logs: Bloc is ${_executionLogBloc == null ? "null" : "closed"}!');
        }
      },
      onClose: () {
        print('🔍 [ExecutionLogButton] Popover closed');
        widget.onOverrideVisibility?.call(false);
        // ✅ 不在这里关闭 Bloc，让它继续存活直到 Widget dispose
      },
      popupBuilder: (context) => _buildExecutionLogPopover(screenSize),
      child: FlowyTooltip(
        message: '查看执行过程',
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onTap: () => _popoverController.show(),
          child: Padding(
            padding: const EdgeInsets.all(4.0),
            child: FlowySvg(
              FlowySvgs.ai_summary_generate_s,
              size: const Size.square(16),
              color: widget.isInHoverBar
                  ? null
                  : Theme.of(context).colorScheme.primary,
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildExecutionLogPopover(Size screenSize) {
    // ⚠️ 如果 Bloc 还未创建或已关闭，显示错误信息
    if (_executionLogBloc == null || _executionLogBloc!.isClosed) {
      print('🔍 [ExecutionLogButton] _buildExecutionLogPopover: bloc is ${_executionLogBloc == null ? "null" : "closed"}');
      return Center(
        child: Text('日志查看器未初始化或已关闭'),
      );
    }
    
    print('🔍 [ExecutionLogButton] _buildExecutionLogPopover: bloc is ready (hashCode: ${_executionLogBloc.hashCode})');
    
    // 🔌 从 ChatAIMessageBloc 中获取真实的 chatId
    final chatId = context.read<ChatAIMessageBloc>().chatId;
    
    // ⚠️ 关键修复：Popover 的 context 是独立的，需要在这里重新提供 Bloc
    return BlocProvider.value(
      value: _executionLogBloc!,
      child: Container(
        padding: const EdgeInsets.all(8),
        child: Column(
          children: [
            // 标题栏
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
              child: Row(
                children: [
                  FlowySvg(
                    FlowySvgs.ai_summary_generate_s,
                    size: const Size.square(16),
                  ),
                  const HSpace(8),
                  FlowyText.medium(
                    '执行过程',
                    fontSize: 14,
                  ),
                  const Spacer(),
                  GestureDetector(
                    onTap: () => _popoverController.close(),
                    child: Padding(
                      padding: const EdgeInsets.all(4),
                      child: FlowySvg(
                        FlowySvgs.close_s,
                        size: const Size.square(12),
                      ),
                    ),
                  ),
                ],
              ),
            ),
            const Divider(height: 1),
            
            // 执行日志查看器
            Expanded(
              child: ExecutionLogViewer(
                sessionId: chatId,
              messageId: widget.message.metadata?[messageQuestionIdKey]?.toString() 
                  ?? widget.message.id,
              height: double.infinity,
              showHeader: false,
            ),
          ),
        ],
      ),
    ),
    );
  }
}
