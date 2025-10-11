import 'package:appflowy/generated/locale_keys.g.dart';
import 'package:appflowy/workspace/application/settings/ai/vector_index_bloc.dart';
import 'package:appflowy_backend/protobuf/flowy-ai/entities.pb.dart';
import 'package:easy_localization/easy_localization.dart';
import 'package:flowy_infra_ui/flowy_infra_ui.dart';
import 'package:flowy_infra_ui/style_widget/button.dart';
import 'package:flowy_infra_ui/style_widget/text.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

/// 向量索引重建设置组件
class VectorIndexSetting extends StatelessWidget {
  const VectorIndexSetting({super.key});

  @override
  Widget build(BuildContext context) {
    return BlocProvider(
      create: (context) => VectorIndexBloc()..add(const VectorIndexEvent.started()),
      child: const _VectorIndexSettingContent(),
    );
  }
}

class _VectorIndexSettingContent extends StatelessWidget {
  const _VectorIndexSettingContent();

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<VectorIndexBloc, VectorIndexState>(
      builder: (context, state) {
        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // 标题和描述
            const _SectionHeader(),
            const VSpace(16),
            
            // 重建按钮和状态
            _RebuildButton(isRunning: state.isRunning),
            const VSpace(16),
            
            // 进度显示
            if (state.status != null && state.status!.state != VectorIndexStatePB.IndexIdle)
              _ProgressSection(status: state.status!),
            
            // 日志显示
            if (state.status != null && state.status!.recentLogs.isNotEmpty)
              ...[
                const VSpace(16),
                _LogsSection(logs: state.status!.recentLogs),
              ],
          ],
        );
      },
    );
  }
}

class _SectionHeader extends StatelessWidget {
  const _SectionHeader();

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        FlowyText.semibold(
          '向量索引管理', // TODO: 添加到 locale_keys
          fontSize: 16,
        ),
        const VSpace(8),
        FlowyText.regular(
          '重建向量索引可以确保所有文档都被正确索引，提高 AI 搜索和 RAG 功能的准确性。',
          fontSize: 12,
          color: Theme.of(context).hintColor,
          maxLines: 3,
        ),
      ],
    );
  }
}

class _RebuildButton extends StatelessWidget {
  const _RebuildButton({required this.isRunning});

  final bool isRunning;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        FlowyTextButton(
          isRunning ? '停止索引' : '重建向量索引',
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          fillColor: isRunning 
            ? Theme.of(context).colorScheme.error
            : Theme.of(context).colorScheme.primary,
          hoverColor: isRunning
            ? Theme.of(context).colorScheme.error.withOpacity(0.8)
            : Theme.of(context).colorScheme.primary.withOpacity(0.8),
          fontColor: Colors.white,
          onPressed: () {
            if (isRunning) {
              context.read<VectorIndexBloc>().add(
                const VectorIndexEvent.stopIndexing(),
              );
            } else {
              context.read<VectorIndexBloc>().add(
                const VectorIndexEvent.rebuildIndex(),
              );
            }
          },
        ),
        if (isRunning) ...[
          const HSpace(12),
          const SizedBox(
            width: 16,
            height: 16,
            child: CircularProgressIndicator(strokeWidth: 2),
          ),
        ],
      ],
    );
  }
}

class _ProgressSection extends StatelessWidget {
  const _ProgressSection({required this.status});

  final VectorIndexStatusPB status;

  @override
  Widget build(BuildContext context) {
    final progress = status.totalDocuments > 0
        ? status.indexedDocuments / status.totalDocuments
        : 0.0;

    String stateText = '';
    Color? stateColor;
    
    switch (status.state) {
      case VectorIndexStatePB.IndexRunning:
        stateText = '索引中...';
        stateColor = Theme.of(context).colorScheme.primary;
        break;
      case VectorIndexStatePB.IndexCompleted:
        stateText = '完成';
        stateColor = Colors.green;
        break;
      case VectorIndexStatePB.IndexFailed:
        stateText = '失败';
        stateColor = Theme.of(context).colorScheme.error;
        break;
      case VectorIndexStatePB.IndexStopping:
        stateText = '停止中...';
        stateColor = Theme.of(context).hintColor;
        break;
      default:
        stateText = '空闲';
    }

    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceVariant,
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              FlowyText.semibold('索引进度', fontSize: 14),
              const Spacer(),
              FlowyText.regular(
                stateText,
                fontSize: 12,
                color: stateColor,
              ),
            ],
          ),
          const VSpace(8),
          LinearProgressIndicator(
            value: progress,
            backgroundColor: Theme.of(context).colorScheme.surface,
            minHeight: 8,
            borderRadius: BorderRadius.circular(4),
          ),
          const VSpace(8),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              FlowyText.regular(
                '${status.indexedDocuments} / ${status.totalDocuments} 个文档',
                fontSize: 12,
                color: Theme.of(context).hintColor,
              ),
              if (status.state == VectorIndexStatePB.IndexRunning)
                FlowyText.regular(
                  '${(progress * 100).toStringAsFixed(1)}%',
                  fontSize: 12,
                  color: Theme.of(context).hintColor,
                ),
            ],
          ),
          if (status.error != null && status.error!.isNotEmpty) ...[
            const VSpace(8),
            Container(
              padding: const EdgeInsets.all(8),
              decoration: BoxDecoration(
                color: Theme.of(context).colorScheme.error.withOpacity(0.1),
                borderRadius: BorderRadius.circular(4),
                border: Border.all(
                  color: Theme.of(context).colorScheme.error.withOpacity(0.3),
                ),
              ),
              child: Row(
                children: [
                  Icon(
                    Icons.error_outline,
                    size: 16,
                    color: Theme.of(context).colorScheme.error,
                  ),
                  const HSpace(8),
                  Expanded(
                    child: FlowyText.regular(
                      status.error!,
                      fontSize: 12,
                      color: Theme.of(context).colorScheme.error,
                      maxLines: 3,
                    ),
                  ),
                ],
              ),
            ),
          ],
        ],
      ),
    );
  }
}

class _LogsSection extends StatelessWidget {
  const _LogsSection({required this.logs});

  final List<String> logs;

  @override
  Widget build(BuildContext context) {
    return Container(
      height: 200,
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceVariant,
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          FlowyText.semibold('索引日志', fontSize: 14),
          const VSpace(8),
          Expanded(
            child: ListView.separated(
              itemCount: logs.length,
              separatorBuilder: (context, index) => const VSpace(4),
              itemBuilder: (context, index) {
                final log = logs[logs.length - 1 - index]; // 倒序显示，最新的在上面
                return FlowyText.regular(
                  log,
                  fontSize: 11,
                  color: Theme.of(context).hintColor,
                  maxLines: 2,
                  fontFamily: 'monospace',
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}

